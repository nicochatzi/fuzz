use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use heapless::spsc::Queue;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use pyo3::prelude::*;
use pyo3::{py_run, PyCell, PyObjectProtocol};
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

    let path = format!("{}/scripts/tri.py", std::env!("CARGO_MANIFEST_DIR"));
    let path_clone = path.clone();

    let watcher_thread = std::thread::spawn(move || {
        let (tx, rx) = channel();

        let mut watcher: RecommendedWatcher =
            Watcher::new(tx, std::time::Duration::from_secs(1)).unwrap();
        watcher.watch(path, RecursiveMode::Recursive).unwrap();

        // This is a simple loop, but you may want to use more complex logic here,
        // for example to handle I/O.
        loop {
            match rx.recv() {
                Ok(event) => msg_tx.enqueue(0).unwrap(),
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });

    let py_thread = std::thread::spawn(move || {
        let _: PyResult<()> = Python::with_gil(move |py| {
            let re_interpret = move || -> PyResult<&PyAny> {
                println!("reinterpreting");
                let code = String::from_utf8(std::fs::read(&path_clone).unwrap()).unwrap();
                let dsp = PyModule::from_code(py, &code, "tri.py", "scripts")?;

                let proc = dsp.getattr("Processor")?.call0()?;
                proc.call_method1("update", (256, 44000))?;
                Ok(proc)
            };

            let mut proc = re_interpret()?;

            loop {
                while producer.len() != producer.capacity() {
                    let buffer: Vec<f32> = proc.call_method0("process")?.extract()?;
                    for sample in buffer {
                        producer.enqueue(sample);
                    }
                }

                if msg_rx.dequeue().is_some() {
                    proc = re_interpret()?;
                }
            }
        });
    });

    // hackday style wait until data is available
    while consumer.peek().is_none() {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    let audio_thread = std::thread::spawn(move || match config.sample_format() {
        cpal::SampleFormat::F32 => {
            let channels = config.channels() as usize;
            let mut next_value = move || consumer.dequeue().unwrap_or_default();

            let stream = device
                .build_output_stream(
                    &config.into(),
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        write_data(data, channels, &mut next_value)
                    },
                    |err| eprintln!("an error occurred on stream: {}", err),
                )
                .unwrap();

            stream.play().unwrap();

            loop {
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
        }
        _ => (),
    });

    audio_thread.join().unwrap();
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&(next_sample() * 0.0));
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
