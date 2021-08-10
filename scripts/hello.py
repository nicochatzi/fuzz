def init():
    print("in init()")


def update(sample_rate):
    print(f"in update() got sample rate : {sample_rate}")


def process(buffer):
    print(f"in process() got buffer : {buffer}")
