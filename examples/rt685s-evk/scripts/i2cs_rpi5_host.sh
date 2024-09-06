#!/bin/bash

BUS  = 1
ADDR = "0x1F"

# read from chip address
read()
{
    local addr=$1

    # format for read request is:
    # 0, ADDR, -, 0xAA

    # response is:
    # MEM[ADDR], 0xAA

    i2ctransfer -y $BUS "w4@$ADDR" 0 $addr 0 0xAA "r2"
}

# write to chip address
write()
{
    local addr=$1
    local data=$2

    # format for write request is:
    # 1, ADDR, DATA, 0xAA

    # response is:
    # 0xAA

    i2ctransfer -y $BUS "w4@$ADDR" 1 $addr $data 0xAA "r1"
}

write 0x00 0x01
read 0x00
