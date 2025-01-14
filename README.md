# embassy-imxrt

[![no-std](https://github.com/pop-project/embassy-imxrt/actions/workflows/nostd.yml/badge.svg)](https://github.com/pop-project/embassy-imxrt/actions/workflows/nostd.yml)
[![check](https://github.com/pop-project/embassy-imxrt/actions/workflows/check.yml/badge.svg)](https://github.com/pop-project/embassy-imxrt/actions/workflows/check.yml)
[![rolling](https://github.com/pop-project/embassy-imxrt/actions/workflows/rolling.yml/badge.svg)](https://github.com/pop-project/embassy-imxrt/actions/workflows/rolling.yml)
[![LICENSE](https://img.shields.io/badge/License-MIT-blue)](./LICENSE)

## Introduction

This is the Embassy HAL for IMXRT family. For development, we will be
focusing on supporting IMXRT6 family first using RT685 dev kit as the
development platform. The plan is to eventually upstream this to
Embassy official repository,

## Peripherals HALs

* ADC
* I2C
* eSPI
  * Need more investigation, ignore for now
* SPI
* GPIO
* Timer
* Clocks
* Interrupt
* RTC
* UART
* Watchdog
* PWM
* DMA

## Plan

We will focus on Asyn HALs first. Blocking HAL can be optional.

Considering various degrees of familiarity with Rust + Embadded HAL,
before you start on a HAL piece, feel free to reach out to Felipe
Balbi, Jerry Xie, Jimi Huard, or Madeleyne to sketch out your ideas.
