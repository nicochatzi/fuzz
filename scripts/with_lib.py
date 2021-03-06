import fuzz


class Processor:
    def __init__(self):
        self.phase_inc = 0
        self.phase = 0
        self.freq = 55
        self.buffer = []

    def update(self, buffer_size, sample_rate):
        self.buffer = [0 for i in range(buffer_size)]
        self.phase_inc = 2 * 3.14 * self.freq / sample_rate

    def process(self):
        for sample in range(len(self.buffer)):
            self.buffer[sample] = fuzz.sin(self.phase)
            self.phase = (self.phase + self.phase_inc) % (2 * 3.14)

        return self.buffer


proc = Processor()
proc.update(16, 44_000)
buf = proc.process()
print(buf)
