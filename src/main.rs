use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use heapless::spsc::Queue;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use pyo3::prelude::*;
use std::sync::mpsc::channel;

fn main() {
    static mut AUDIO_QUEUE: Queue<f32, 1024> = Queue::new();
    let (mut producer, mut consumer) = unsafe { AUDIO_QUEUE.split() };

    static mut MESSAGE_QUEUE: Queue<u8, 8> = Queue::new();
    let (mut msg_tx, mut msg_rx) = unsafe { MESSAGE_QUEUE.split() };

    let device = cpal::default_host()
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();
    let sample_rate = config.sample_rate().0;

    let path = format!("{}/scripts/tri.py", std::env!("CARGO_MANIFEST_DIR"));
    let path_clone = path.clone();

    // watch a file and enqueue a message when the file has changed
    let _watcher_thread = std::thread::spawn(move || {
        let (tx, rx) = channel();

        let mut watcher: RecommendedWatcher =
            Watcher::new(tx, std::time::Duration::from_secs(1)).unwrap();

        watcher.watch(path, RecursiveMode::Recursive).unwrap();

        loop {
            match rx.recv() {
                Ok(_) => msg_tx.enqueue(0).unwrap(),
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });

    // start the embedded python interpreter and enqueue samples
    let _py_thread = std::thread::spawn(move || {
        let _: PyResult<()> = Python::with_gil(move |py| {
            let reinterpret = move || -> PyResult<&PyAny> {
                println!("reinterpreting");
                let code = String::from_utf8(std::fs::read(&path_clone).unwrap()).unwrap();
                let dsp = PyModule::from_code(py, &code, "tri.py", "scripts")?;

                let proc = dsp.getattr("Processor")?.call0()?;
                proc.call_method1("update", (256, sample_rate))?;
                Ok(proc)
            };

            let mut proc = reinterpret()?;

            loop {
                while producer.len() != producer.capacity() {
                    let buffer: Vec<f32> = proc.call_method0("process")?.extract()?;
                    for sample in buffer {
                        let _ = producer.enqueue(sample);
                    }
                }

                if msg_rx.dequeue().is_some() {
                    proc = reinterpret()?;
                }
            }
        });
    });

    // hackday style wait until data is available
    while consumer.peek().is_none() {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    // start the audio thread and dequeue samples
    let audio_thread = std::thread::spawn(move || match config.sample_format() {
        cpal::SampleFormat::F32 => {
            fn write_data<T: cpal::Sample>(
                out: &mut [T],
                chans: usize,
                next_sample: &mut dyn FnMut() -> f32,
            ) {
                for frame in out.chunks_mut(chans) {
                    let value: T = cpal::Sample::from::<f32>(&(next_sample()));
                    for sample in frame.iter_mut() {
                        *sample = value;
                    }
                }
            }

            let channels = config.channels() as usize;
            let mut gen_sample = move || consumer.dequeue().unwrap_or_default();

            let stream = device
                .build_output_stream(
                    &config.into(),
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        write_data(data, channels, &mut gen_sample)
                    },
                    |err| eprintln!("an error occurred on stream: {}", err),
                )
                .unwrap();

            stream.play().unwrap();

            loop {
                std::thread::sleep(std::time::Duration::from_secs(1_000_000));
            }
        }
        _ => (),
    });

    audio_thread.join().unwrap();
}
