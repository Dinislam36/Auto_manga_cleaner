import onnxruntime as onnx
import time
import numpy as np
from tqdm import tqdm as tqdm


onnx_model = onnx.InferenceSession('model.onnx')

# measure inference time
ITERATIONS = 2

input = np.random.rand(1, 3, 1176, 828).astype(np.float32)

start = time.time()
for i in tqdm(range(ITERATIONS)):
    onnx_model.run(None, {'input': input})
end = time.time()

print(f"Average inference time: {(end - start) / ITERATIONS}s", )
