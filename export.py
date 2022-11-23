#!/usr/bin/env python3
import torch
import torch.nn as nn
import torchvision
import argparse
from pathlib import Path

IMAGE_SIZE_W = 828
IMAGE_SIZE_H = 1176

class JapaneseTextDetector(nn.Module):
    def __convBlock(self, ch_in, ch_out, padding, kernel_size):
        conv = nn.Sequential(
            nn.Conv2d(ch_in, ch_out, padding=padding, kernel_size=kernel_size),
            nn.BatchNorm2d(ch_out),
            nn.ReLU(),
            
            nn.Conv2d(ch_out, ch_out, kernel_size=(1, 1)),
            nn.BatchNorm2d(ch_out),
            nn.ReLU()
        )
        
        return conv
    
    def __init__(self, img_h, img_w):
        super(JapaneseTextDetector, self).__init__()
        
        self.img_h = img_h
        self.img_w = img_w
        
        self.conv_block_1 = self.__convBlock(1, 16, padding=(1, 1), kernel_size=(3, 3)) 
        self.max_pool_1 = nn.MaxPool2d((2, 2)) # 1170x826 -> 585x413
        
        self.conv_block_2 = self.__convBlock(16, 32, padding=(1, 1), kernel_size=(3, 3)) 
        self.max_pool_2 = nn.MaxPool2d((2, 2)) # 585x413 -> 292x206
        
        self.conv_block_3 = self.__convBlock(32, 64, padding=(1, 1), kernel_size=(3, 3)) 
        self.max_pool_3 = nn.MaxPool2d((2, 2)) # 292x206 -> 146x103
        
        self.conv_block_4 = self.__convBlock(64, 128, padding=(1, 1), kernel_size=(3, 3)) 
        self.max_pool_4 = nn.MaxPool2d((2, 2)) # 146x103 -> 73x51
        
        self.deconv_1 = nn.ConvTranspose2d(128, 64, kernel_size=(3, 3), stride=(2, 2), padding=(1, 1), output_padding=(1, 1))
        
        self.conv_filter_1 = nn.Conv2d(128, 64, kernel_size=(1, 1))
        self.conv_block_5 = self.__convBlock(64, 64, padding=(1, 1), kernel_size=(3, 3)) 
        
        self.deconv_2 = nn.ConvTranspose2d(64, 32, kernel_size=(3, 3), stride=(2, 2), padding=(1, 1), output_padding=(1, 1))
        
        self.conv_filter_2 = nn.Conv2d(64, 32, kernel_size=(1, 1))
        self.conv_block_6 = self.__convBlock(32, 32, padding=(1, 1), kernel_size=(3, 3)) 
        
        self.deconv_3 = nn.ConvTranspose2d(32, 16, kernel_size=(3, 3), stride=(2, 2), padding=(1, 0), output_padding=(1, 0))
        
        self.conv_block_7 = nn.Conv2d(32, 1, padding=(1, 1), kernel_size=(3, 3))
        
    def forward(self, x):
        c1 = self.conv_block_1(x)
        x = self.max_pool_1(c1)
        
        c2 = self.conv_block_2(x)
        x = self.max_pool_2(c2)
        
        c3 = self.conv_block_3(x)
        x = self.max_pool_3(c3)
        
        x = self.conv_block_4(x)
        
        x = self.deconv_1(x)
        
        x = torch.cat((c3, x), 1)
        x = self.conv_filter_1(x)
        
        x = self.conv_block_5(x)
        x = self.deconv_2(x)
        
        x = torch.cat((c2, x), 1)
        x = self.conv_filter_2(x)
        
        x = self.conv_block_6(x)
        x = self.deconv_3(x)
        
        x = torch.cat((c1, x), 1)
        x = self.conv_block_7(x)

        return x

class Resnet34(nn.Module):
    def __init__(self):
        super(Resnet34, self).__init__()
        
        resnet34 = torchvision.models.resnet34()
        
        self.conv1 = resnet34.conv1
        self.maxpool = resnet34.maxpool
        self.layer1 = resnet34.layer1
        self.layer2 = resnet34.layer2
        self.layer3 = resnet34.layer3
        self.layer4 = resnet34.layer4
        
    def forward(self, x):
        x = self.conv1(x)
        e1 = self.maxpool(x)
        e1 = self.layer1(e1)
        e2 = self.layer2(e1)
        e3 = self.layer3(e2)
        e4 = self.layer4(e3)
        
        return x, e1, e2, e3, e4

class PretrainedTextDetector(nn.Module):
    def __doubleConv(self, ch_in, ch_out, padding, kernel_size):
        conv = nn.Sequential(
            nn.Conv2d(ch_in, ch_out, padding=padding, kernel_size=kernel_size),
            nn.BatchNorm2d(ch_out),
            nn.ReLU(),
            
            nn.Conv2d(ch_out, ch_out, padding=padding, kernel_size=kernel_size),
            nn.BatchNorm2d(ch_out),
            nn.ReLU(),
        )
        
        return conv
    
    def __init__(self, img_h, img_w, finetune=False):
        super(PretrainedTextDetector, self).__init__()

        self.img_h = img_h
        self.img_w = img_w
        self.finetune = finetune
        self.resnet = Resnet34()
        self.resnet.eval()
        
        self.conv_1 = self.__doubleConv(512, 512, (1, 1), (3, 3))
        self.deconv_1 = nn.ConvTranspose2d(512, 256, kernel_size=(3, 3), stride=(2, 2), padding=(1, 1), output_padding=(1, 1))
        self.conv_2 = self.__doubleConv(512, 256, (1, 1), (3, 3))
        self.deconv_2 = nn.ConvTranspose2d(256, 128, kernel_size=(3, 3), stride=(2, 2), padding=(1, 1), output_padding=(0, 1))
        self.conv_3 = self.__doubleConv(256, 128, (1, 1), (3, 3))
        self.deconv_3 = nn.ConvTranspose2d(128, 64, kernel_size=(3, 3), stride=(2, 2), padding=(1, 1), output_padding=(1, 0))
        self.conv_4 = self.__doubleConv(128, 64, (1, 1), (3, 3))
        self.deconv_4 = nn.ConvTranspose2d(64, 32, kernel_size=(3, 3), stride=(2, 2), padding=(1, 1), output_padding=(1, 1))
        self.conv_5 = self.__doubleConv(96, 64, (1, 1), (3, 3))
        self.deconv_5 = nn.ConvTranspose2d(64, 32, kernel_size=(3, 3), stride=(2, 2), padding=(1, 1), output_padding=(1, 1))
        self.conv_6 = self.__doubleConv(32, 16, (1, 1), (3, 3))
        self.out_conv = nn.Conv2d(16, 1, padding=(1, 1), kernel_size=(3, 3))
        
        for param in self.parameters():
            param.requires_grad = True

    def forward(self, x):
        if self.finetune:
            e0, e1, e2, e3, e4 = self.resnet(x)
        else:
            with torch.no_grad():
                e0, e1, e2, e3, e4 = self.resnet(x)
        x = self.conv_1(e4)
        x = self.deconv_1(x)
        x = torch.cat((x, e3), 1)
        x = self.conv_2(x)
        x = self.deconv_2(x)
        x = torch.cat((x, e2), 1)
        x = self.conv_3(x)
        x = self.deconv_3(x)
        x = torch.cat((x, e1), 1)
        x = self.conv_4(x)
        x = self.deconv_4(x)
        x = torch.cat((x, e0), 1)
        x = self.conv_5(x)
        x = self.deconv_5(x)
        x = self.conv_6(x)
        x = self.out_conv(x)
        return x

parser = argparse.ArgumentParser()
parser.add_argument('model_state', type=Path)

args = parser.parse_args()

model = PretrainedTextDetector(IMAGE_SIZE_W, IMAGE_SIZE_H, finetune=True)
model.load_state_dict(torch.load(args.model_state))
model.eval()

random_data = torch.rand(1, 3, IMAGE_SIZE_H, IMAGE_SIZE_W)

# test the shape
model(random_data)

# convert to TorchScript
model_jit = torch.jit.script(model)

output_filename = args.model_state.with_suffix('.onnx')

print("Saving model to", output_filename)

torch.onnx.export(model_jit, random_data, output_filename, verbose=False, opset_version=12, input_names=['input'], output_names=['output'])

