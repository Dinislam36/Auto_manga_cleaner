import onnx
import onnxoptimizer as optimizer
import argparse
from pathlib import Path


parser = argparse.ArgumentParser()
parser.add_argument('model_onnx', type=Path)
args = parser.parse_args()

src_onnx = args.model_onnx
opt_onnx = src_onnx.with_name(src_onnx.stem + '_opt' + src_onnx.suffix)

# load model
model = onnx.load(src_onnx)

# optimize
passes = [
    # 'rename_input_output', 
    # 'set_unique_name_for_nodes', 'nop',
    # 'eliminate_nop_cast', 'eliminate_nop_dropout', 'eliminate_nop_flatten',
    # 'extract_constant_to_initializer', 'eliminate_if_with_const_cond',
    # 'eliminate_nop_monotone_argmax', 'eliminate_nop_pad', 'eliminate_nop_concat',
    # 'eliminate_nop_split', 'eliminate_nop_expand', 'eliminate_shape_gather',
    # 'eliminate_slice_after_shape', 'eliminate_nop_transpose', 'fuse_add_bias_into_conv',
    # 'fuse_bn_into_conv', 'fuse_consecutive_concats', 'fuse_consecutive_log_softmax',
    # 'fuse_consecutive_reduce_unsqueeze', 'fuse_consecutive_squeezes',
    # 'fuse_consecutive_transposes', 'fuse_matmul_add_bias_into_gemm',
    # 'fuse_pad_into_conv', 'fuse_pad_into_pool', 'fuse_transpose_into_gemm',
    # 'replace_einsum_with_matmul',
    # 'lift_lexical_references',
    # 'split_init',
    # 'split_predict', 'fuse_concat_into_reshape', 'eliminate_nop_reshape',
    # 'eliminate_deadend', 'eliminate_identity', 'eliminate_shape_op',
    # 'eliminate_unused_initializer', 'eliminate_duplicate_initializer'
]
print("Running optimization passes: {}".format(passes))
model = optimizer.optimize(model, passes, fixed_point=False)

# save optimized model
with open(opt_onnx, "wb") as f:
    f.write(model.SerializeToString())