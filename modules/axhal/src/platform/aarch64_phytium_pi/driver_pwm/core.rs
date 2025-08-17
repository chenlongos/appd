use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m::asm;
use super::{regs::{PwmRegisters, DBCTRL, DBDLY, TIM_CNT, TIM_CTRL, STATE, PWM_PERIOD, PWM_CTRL, PWM_CCR}, types::*};

impl PwmController {
    pub unsafe fn new(base_addr: usize) -> Self {
        Self {
            base: base_addr,
            channels: [
                PwmChannel { config: None, enabled: false },
                PwmChannel { config: None, enabled: false }
            ],
        }
    }

    fn registers(&self) -> &PwmRegisters {
        unsafe { &*(self.base as *const PwmRegisters) }
    }

    fn get_channel_reg(&self, channel: usize) -> ChannelRegisters {
        let regs = self.registers();
        match channel {
            0 => ChannelRegisters {
                tim_cnt: &regs.ch0_tim_cnt,
                tim_ctrl: &regs.ch0_tim_ctrl,
                state: &regs.ch0_state,
                pwm_period: &regs.ch0_pwm_period,
                pwm_ctrl: &regs.ch0_pwm_ctrl,
                pwm_ccr: &regs.ch0_pwm_ccr,
            },
            1 => ChannelRegisters {
                tim_cnt: &regs.ch1_tim_cnt,
                tim_ctrl: &regs.ch1_tim_ctrl,
                state: &regs.ch1_state,
                pwm_period: &regs.ch1_pwm_period,
                pwm_ctrl: &regs.ch1_pwm_ctrl,
                pwm_ccr: &regs.ch1_pwm_ccr,
            },
            _ => unreachable!(),
        }
    }

    pub fn configure_channel(&mut self, channel: usize, config: PwmConfig) -> Result<(), &'static str> {
        if channel >= CHANNELS_PER_CONTROLLER {
            return Err("Invalid channel number");
        }
        if config.duty_cycle > 1.0 || config.duty_cycle < 0.0 {
            return Err("Duty cycle must be between 0.0 and 1.0");
        }
        self.disable_channel(channel);
        let div = (SYSTEM_CLK / config.frequency) as u16;
        let period_cycles = (SYSTEM_CLK as f32 / (div as f32 * config.frequency as f32)) as u32;
        let period_reg = period_cycles.checked_sub(1).ok_or("Period too small")?;
        if period_reg > 0xFFFF {
            return Err("Period value too large");
        }
        let duty_cycles = (period_reg as f32 * config.duty_cycle) as u16;
        let regs = self.registers();
        if let Some(deadtime) = config.deadtime_ns {
            let delay_cycles = (deadtime as f32 * SYSTEM_CLK as f32 / 1e9) as u16;
            let delay_cycles = delay_cycles.min((1 << 10) - 1);
            regs.dbdly.write(DBDLY::DBRED.val(delay_cycles) + DBDLY::DBFED.val(delay_cycles));
            regs.dbctrl.modify(DBCTRL::OUT_MODE::FullDeadband + DBCTRL::IN_MODE::PWM0 + DBCTRL::POLSEL::AH);
        }
        let ch_reg = self.get_channel_reg(channel);
        ch_reg.tim_ctrl.modify(TIM_CTRL::DIV.val(div.into()) + TIM_CTRL::MODE.val(config.counting_mode) + TIM_CTRL::ENABLE::Disabled);
        ch_reg.pwm_period.modify(PWM_PERIOD::CCR.val(period_reg as u16));
        ch_reg.pwm_ctrl.modify(PWM_CTRL::MODE::Compare + PWM_CTRL::DUTY_SEL.val(config.use_fifo as u32) + PWM_CTRL::ICOV.val(config.initial_value) + PWM_CTRL::CMP.val(config.output_behavior) + PWM_CTRL::IE::SET);
        if config.use_fifo {
            for _ in 0..4 {
                ch_reg.pwm_ccr.write(PWM_CCR::CCR.val(duty_cycles));
            }
            ch_reg.pwm_ctrl.modify(PWM_CTRL::FIFO_EMPTY_ENABLE::SET);
        } else {
            ch_reg.pwm_ccr.write(PWM_CCR::CCR.val(duty_cycles));
        }
        self.channels[channel].config = Some(config);
        Ok(())
    }

    pub fn enable_channel(&mut self, channel: usize) -> Result<(), &'static str> {
        if channel >= CHANNELS_PER_CONTROLLER {
            return Err("Invalid channel number");
        }
        let ch_reg = self.get_channel_reg(channel);
        ch_reg.tim_ctrl.modify(TIM_CTRL::ENABLE::SET);
        self.channels[channel].enabled = true;
        Ok(())
    }

    pub fn disable_channel(&mut self, channel: usize) {
        let ch_reg = self.get_channel_reg(channel);
        ch_reg.tim_ctrl.modify(TIM_CTRL::ENABLE::CLEAR);
        self.channels[channel].enabled = false;
    }

    pub fn safe_stop_channel(&mut self, channel: usize) -> Result<(), &'static str> {
        let ch_reg = self.get_channel_reg(channel);
        ch_reg.pwm_ccr.write(PWM_CCR::CCR.val(0));
        while ch_reg.tim_cnt.read(TIM_CNT::CNT) != 0 {
            asm::nop();
        }
        self.disable_channel(channel);
        Ok(())
    }

    pub fn push_fifo_data(&mut self, channel: usize, duty_value: u16) -> Result<(), &'static str> {
        if channel >= CHANNELS_PER_CONTROLLER {
            return Err("Invalid channel number");
        }
        let ch_reg = self.get_channel_reg(channel);
        if ch_reg.state.matches_all(STATE::FIFO_FULL::SET) {
            return Err("FIFO full");
        }
        ch_reg.pwm_ccr.write(PWM_CCR::CCR.val(duty_value));
        Ok(())
    }

    pub fn handle_interrupt(&mut self) {
        for channel in 0..CHANNELS_PER_CONTROLLER {
            if let Err(e) = self.handle_channel_interrupt(channel) {
                // log::error!("PWM ch{} error: {}", channel, e);
            }
        }
    }

    fn handle_channel_interrupt(&mut self, channel: usize) -> Result<(), &'static str> {
        let ch_reg = self.get_channel_reg(channel);
        let state = ch_reg.state.get();
        if state & STATE::FIFO_EMPTY.mask != 0 {
            if ch_reg.tim_cnt.read(TIM_CNT::CNT) == 0 {
                if let Some(config) = &self.channels[channel].config {
                    let period = ch_reg.pwm_period.read(PWM_PERIOD::CCR) + 1;
                    let duty_cycles = (period as f32 * config.duty_cycle) as u16;
                    self.push_fifo_data(channel, duty_cycles)?;
                }
            }
            ch_reg.state.write(STATE::FIFO_EMPTY::SET);
        }
        if state & STATE::OVFIF.mask != 0 {
            ch_reg.state.write(STATE::OVFIF::SET);
        }
        if state & STATE::CHIF.mask != 0 {
            ch_reg.state.modify(STATE::CHIF::SET);
        }
        Ok(())
    }
}

struct ChannelRegisters<'a> {
    tim_cnt: &'a ReadWrite<u32, TIM_CNT::Register>,
    tim_ctrl: &'a ReadWrite<u32, TIM_CTRL::Register>,
    state: &'a ReadWrite<u32, STATE::Register>,
    pwm_period: &'a ReadWrite<u32, PWM_PERIOD::Register>,
    pwm_ctrl: &'a ReadWrite<u32, PWM_CTRL::Register>,
    pwm_ccr: &'a ReadWrite<u32, PWM_CCR::Register>,
}

impl PwmSystem {
    pub fn new() -> Self {
        const CONTROLLER_BASES: [usize; PWM_CONTROLLERS] = [
            0x2804_A000, 0x2804_B000, 0x2804_C000, 0x2804_D000,
            0x2804_E000, 0x2804_F000, 0x2805_0000, 0x2805_1000,
        ];
        let controllers = CONTROLLER_BASES.map(|base| unsafe { PwmController::new(base) });
        Self { controllers }
    }

    pub fn global_enable(&self) {
        let mut enable_mask: u32 = 0;
        for (i, ctrl) in self.controllers.iter().enumerate() {
            if ctrl.channels.iter().any(|ch| ch.config.is_some()) {
                enable_mask |= 1 << i;
            }
        }
        unsafe {
            let reg_ptr = GLOBAL_ENABLE_REG_ADDR as *mut u32;
            reg_ptr.write_volatile(enable_mask);
        }
    }

    pub fn controller(&mut self, index: usize) -> Option<&mut PwmController> {
        if index < PWM_CONTROLLERS {
            Some(&mut self.controllers[index])
        } else {
            None
        }
    }
}