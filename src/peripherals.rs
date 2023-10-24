use core::mem::transmute;

use cortex_m::peripheral::*;
use teensy4_bsp::ral::*;

// This module provides safe access to teensy board peripherals and cortex-m core
// peripherals without codegen bloat, provided the following is upheld:
// - CPU interrupts cannot change the execution flow, and there are no other cores
//  on the machine.

// teensy board peripherals
pub const fn adc1() -> adc::ADC1 {
    unsafe { adc::ADC1::instance() }
}

pub const fn adc2() -> adc::ADC2 {
    unsafe { adc::ADC2::instance() }
}

pub const fn adc_etc() -> adc_etc::ADC_ETC {
    unsafe { adc_etc::ADC_ETC::instance() }
}

pub const fn aipstz1() -> aipstz::AIPSTZ1 {
    unsafe { aipstz::AIPSTZ1::instance() }
}

pub const fn aipstz2() -> aipstz::AIPSTZ2 {
    unsafe { aipstz::AIPSTZ2::instance() }
}

pub const fn aipstz3() -> aipstz::AIPSTZ3 {
    unsafe { aipstz::AIPSTZ3::instance() }
}

pub const fn aipstz4() -> aipstz::AIPSTZ4 {
    unsafe { aipstz::AIPSTZ4::instance() }
}

pub const fn aoi1() -> aoi::AOI1 {
    unsafe { aoi::AOI1::instance() }
}

pub const fn aoi2() -> aoi::AOI2 {
    unsafe { aoi::AOI2::instance() }
}

pub const fn bee() -> bee::BEE {
    unsafe { bee::BEE::instance() }
}

pub const fn can1() -> can::CAN1 {
    unsafe { can::CAN1::instance() }
}

pub const fn can2() -> can::CAN2 {
    unsafe { can::CAN2::instance() }
}

pub const fn can3() -> can3::CAN3 {
    unsafe { can3::CAN3::instance() }
}

pub const fn ccm() -> ccm::CCM {
    unsafe { ccm::CCM::instance() }
}

pub const fn ccm_analog() -> ccm_analog::CCM_ANALOG {
    unsafe { ccm_analog::CCM_ANALOG::instance() }
}

pub const fn cmp1() -> cmp::CMP1 {
    unsafe { cmp::CMP1::instance() }
}

pub const fn cmp2() -> cmp::CMP2 {
    unsafe { cmp::CMP2::instance() }
}

pub const fn cmp3() -> cmp::CMP3 {
    unsafe { cmp::CMP3::instance() }
}

pub const fn cmp4() -> cmp::CMP4 {
    unsafe { cmp::CMP4::instance() }
}

pub const fn csi() -> csi::CSI {
    unsafe { csi::CSI::instance() }
}

pub const fn csu() -> csu::CSU {
    unsafe { csu::CSU::instance() }
}

pub const fn dcdc() -> dcdc::DCDC {
    unsafe { dcdc::DCDC::instance() }
}

pub const fn dcp() -> dcp::DCP {
    unsafe { dcp::DCP::instance() }
}

pub const fn dma() -> dma::DMA {
    unsafe { dma::DMA::instance() }
}

pub const fn dmamux() -> dmamux::DMAMUX {
    unsafe { dmamux::DMAMUX::instance() }
}

pub const fn enc1() -> enc::ENC1 {
    unsafe { enc::ENC1::instance() }
}

pub const fn enc2() -> enc::ENC2 {
    unsafe { enc::ENC2::instance() }
}

pub const fn enc3() -> enc::ENC3 {
    unsafe { enc::ENC3::instance() }
}

pub const fn enc4() -> enc::ENC4 {
    unsafe { enc::ENC4::instance() }
}

pub const fn enet1() -> enet::ENET1 {
    unsafe { enet::ENET1::instance() }
}

pub const fn enet2() -> enet::ENET2 {
    unsafe { enet::ENET2::instance() }
}

pub const fn ewm() -> ewm::EWM {
    unsafe { ewm::EWM::instance() }
}

pub const fn flexio1() -> flexio::FLEXIO1 {
    unsafe { flexio::FLEXIO1::instance() }
}

pub const fn flexio2() -> flexio::FLEXIO2 {
    unsafe { flexio::FLEXIO2::instance() }
}

pub const fn flexio3() -> flexio::FLEXIO3 {
    unsafe { flexio::FLEXIO3::instance() }
}

pub const fn flexram() -> flexram::FLEXRAM {
    unsafe { flexram::FLEXRAM::instance() }
}

pub const fn flexspi1() -> flexspi::FLEXSPI1 {
    unsafe { flexspi::FLEXSPI1::instance() }
}

pub const fn flexspi2() -> flexspi::FLEXSPI2 {
    unsafe { flexspi::FLEXSPI2::instance() }
}

pub const fn gpc() -> gpc::GPC {
    unsafe { gpc::GPC::instance() }
}

pub const fn gpio1() -> gpio::GPIO1 {
    unsafe { gpio::GPIO1::instance() }
}

pub const fn gpio5() -> gpio::GPIO5 {
    unsafe { gpio::GPIO5::instance() }
}

pub const fn gpio2() -> gpio::GPIO2 {
    unsafe { gpio::GPIO2::instance() }
}

pub const fn gpio3() -> gpio::GPIO3 {
    unsafe { gpio::GPIO3::instance() }
}

pub const fn gpio4() -> gpio::GPIO4 {
    unsafe { gpio::GPIO4::instance() }
}

pub const fn gpio6() -> gpio::GPIO6 {
    unsafe { gpio::GPIO6::instance() }
}

pub const fn gpio7() -> gpio::GPIO7 {
    unsafe { gpio::GPIO7::instance() }
}

pub const fn gpio8() -> gpio::GPIO8 {
    unsafe { gpio::GPIO8::instance() }
}

pub const fn gpio9() -> gpio::GPIO9 {
    unsafe { gpio::GPIO9::instance() }
}

pub const fn gpt1() -> gpt::GPT1 {
    unsafe { gpt::GPT1::instance() }
}

pub const fn gpt2() -> gpt::GPT2 {
    unsafe { gpt::GPT2::instance() }
}

pub const fn iomuxc() -> iomuxc::IOMUXC {
    unsafe { iomuxc::IOMUXC::instance() }
}

pub const fn iomuxc_gpr() -> iomuxc_gpr::IOMUXC_GPR {
    unsafe { iomuxc_gpr::IOMUXC_GPR::instance() }
}

pub const fn iomuxc_snvs() -> iomuxc_snvs::IOMUXC_SNVS {
    unsafe { iomuxc_snvs::IOMUXC_SNVS::instance() }
}

pub const fn iomuxc_snvs_gpr() -> iomuxc_snvs_gpr::IOMUXC_SNVS_GPR {
    unsafe { iomuxc_snvs_gpr::IOMUXC_SNVS_GPR::instance() }
}

pub const fn kpp() -> kpp::KPP {
    unsafe { kpp::KPP::instance() }
}

pub const fn lcdif() -> lcdif::LCDIF {
    unsafe { lcdif::LCDIF::instance() }
}

pub const fn lpi2c1() -> lpi2c::LPI2C1 {
    unsafe { lpi2c::LPI2C1::instance() }
}

pub const fn lpi2c2() -> lpi2c::LPI2C2 {
    unsafe { lpi2c::LPI2C2::instance() }
}

pub const fn lpi2c3() -> lpi2c::LPI2C3 {
    unsafe { lpi2c::LPI2C3::instance() }
}

pub const fn lpi2c4() -> lpi2c::LPI2C4 {
    unsafe { lpi2c::LPI2C4::instance() }
}

pub const fn lpspi1() -> lpspi::LPSPI1 {
    unsafe { lpspi::LPSPI1::instance() }
}

pub const fn lpspi2() -> lpspi::LPSPI2 {
    unsafe { lpspi::LPSPI2::instance() }
}

pub const fn lpspi3() -> lpspi::LPSPI3 {
    unsafe { lpspi::LPSPI3::instance() }
}

pub const fn lpspi4() -> lpspi::LPSPI4 {
    unsafe { lpspi::LPSPI4::instance() }
}

pub const fn lpuart1() -> lpuart::LPUART1 {
    unsafe { lpuart::LPUART1::instance() }
}

pub const fn lpuart2() -> lpuart::LPUART2 {
    unsafe { lpuart::LPUART2::instance() }
}

pub const fn lpuart3() -> lpuart::LPUART3 {
    unsafe { lpuart::LPUART3::instance() }
}

pub const fn lpuart4() -> lpuart::LPUART4 {
    unsafe { lpuart::LPUART4::instance() }
}

pub const fn lpuart5() -> lpuart::LPUART5 {
    unsafe { lpuart::LPUART5::instance() }
}

pub const fn lpuart6() -> lpuart::LPUART6 {
    unsafe { lpuart::LPUART6::instance() }
}

pub const fn lpuart7() -> lpuart::LPUART7 {
    unsafe { lpuart::LPUART7::instance() }
}

pub const fn lpuart8() -> lpuart::LPUART8 {
    unsafe { lpuart::LPUART8::instance() }
}

pub const fn ocotp() -> ocotp::OCOTP {
    unsafe { ocotp::OCOTP::instance() }
}

pub const fn pgc() -> pgc::PGC {
    unsafe { pgc::PGC::instance() }
}

pub const fn pit() -> pit::PIT {
    unsafe { pit::PIT::instance() }
}

pub const fn pmu() -> pmu::PMU {
    unsafe { pmu::PMU::instance() }
}

pub const fn pwm1() -> pwm::PWM1 {
    unsafe { pwm::PWM1::instance() }
}

pub const fn pwm2() -> pwm::PWM2 {
    unsafe { pwm::PWM2::instance() }
}

pub const fn pwm3() -> pwm::PWM3 {
    unsafe { pwm::PWM3::instance() }
}

pub const fn pwm4() -> pwm::PWM4 {
    unsafe { pwm::PWM4::instance() }
}

pub const fn pxp() -> pxp::PXP {
    unsafe { pxp::PXP::instance() }
}

pub const fn romc() -> romc::ROMC {
    unsafe { romc::ROMC::instance() }
}

pub const fn rtwdog() -> rtwdog::RTWDOG {
    unsafe { rtwdog::RTWDOG::instance() }
}

pub const fn sai1() -> sai::SAI1 {
    unsafe { sai::SAI1::instance() }
}

pub const fn sai2() -> sai::SAI2 {
    unsafe { sai::SAI2::instance() }
}

pub const fn sai3() -> sai::SAI3 {
    unsafe { sai::SAI3::instance() }
}

pub const fn semc() -> semc::SEMC {
    unsafe { semc::SEMC::instance() }
}

pub const fn snvs() -> snvs::SNVS {
    unsafe { snvs::SNVS::instance() }
}

pub const fn spdif() -> spdif::SPDIF {
    unsafe { spdif::SPDIF::instance() }
}

pub const fn src() -> src::SRC {
    unsafe { src::SRC::instance() }
}

pub const fn tempmon() -> tempmon::TEMPMON {
    unsafe { tempmon::TEMPMON::instance() }
}

pub const fn tmr1() -> tmr::TMR1 {
    unsafe { tmr::TMR1::instance() }
}

pub const fn tmr2() -> tmr::TMR2 {
    unsafe { tmr::TMR2::instance() }
}

pub const fn tmr3() -> tmr::TMR3 {
    unsafe { tmr::TMR3::instance() }
}

pub const fn tmr4() -> tmr::TMR4 {
    unsafe { tmr::TMR4::instance() }
}

pub const fn trng() -> trng::TRNG {
    unsafe { trng::TRNG::instance() }
}

pub const fn tsc() -> tsc::TSC {
    unsafe { tsc::TSC::instance() }
}

pub const fn usb1() -> usb::USB1 {
    unsafe { usb::USB1::instance() }
}

pub const fn usb2() -> usb::USB2 {
    unsafe { usb::USB2::instance() }
}

pub const fn usb_analog() -> usb_analog::USB_ANALOG {
    unsafe { usb_analog::USB_ANALOG::instance() }
}

pub const fn usbnc1() -> usbnc::USBNC1 {
    unsafe { usbnc::USBNC1::instance() }
}

pub const fn usbnc2() -> usbnc::USBNC2 {
    unsafe { usbnc::USBNC2::instance() }
}

pub const fn usbphy1() -> usbphy::USBPHY1 {
    unsafe { usbphy::USBPHY1::instance() }
}

pub const fn usbphy2() -> usbphy::USBPHY2 {
    unsafe { usbphy::USBPHY2::instance() }
}

pub const fn usdhc1() -> usdhc::USDHC1 {
    unsafe { usdhc::USDHC1::instance() }
}

pub const fn usdhc2() -> usdhc::USDHC2 {
    unsafe { usdhc::USDHC2::instance() }
}

pub const fn wdog1() -> wdog::WDOG1 {
    unsafe { wdog::WDOG1::instance() }
}

pub const fn wdog2() -> wdog::WDOG2 {
    unsafe { wdog::WDOG2::instance() }
}

pub const fn xbara1() -> xbara1::XBARA1 {
    unsafe { xbara1::XBARA1::instance() }
}

pub const fn xbarb2() -> xbarb::XBARB2 {
    unsafe { xbarb::XBARB2::instance() }
}

pub const fn xbarb3() -> xbarb::XBARB3 {
    unsafe { xbarb::XBARB3::instance() }
}

pub const fn xtalosc24m() -> xtalosc24m::XTALOSC24M {
    unsafe { xtalosc24m::XTALOSC24M::instance() }
}

// cortex-m core peripherals
pub const fn ac() -> AC {
    unsafe { transmute::<_, cortex_m::Peripherals>(()).AC }
}
pub const fn cbp() -> CBP {
    unsafe { transmute::<_, cortex_m::Peripherals>(()).CBP }
}
pub const fn cpuid() -> CPUID {
    unsafe { transmute::<_, cortex_m::Peripherals>(()).CPUID }
}
pub const fn dcb() -> DCB {
    unsafe { transmute::<_, cortex_m::Peripherals>(()).DCB }
}
pub const fn dwt() -> DWT {
    unsafe { transmute::<_, cortex_m::Peripherals>(()).DWT }
}
pub const fn fpb() -> FPB {
    unsafe { transmute::<_, cortex_m::Peripherals>(()).FPB }
}
pub const fn fpu() -> FPU {
    unsafe { transmute::<_, cortex_m::Peripherals>(()).FPU }
}
pub const fn icb() -> ICB {
    unsafe { transmute::<_, cortex_m::Peripherals>(()).ICB }
}
pub const fn itm() -> ITM {
    unsafe { transmute::<_, cortex_m::Peripherals>(()).ITM }
}
pub const fn mpu() -> MPU {
    unsafe { transmute::<_, cortex_m::Peripherals>(()).MPU }
}
pub const fn nvic() -> NVIC {
    unsafe { transmute::<_, cortex_m::Peripherals>(()).NVIC }
}
pub const fn sau() -> SAU {
    unsafe { transmute::<_, cortex_m::Peripherals>(()).SAU }
}
pub const fn scb() -> SCB {
    unsafe { transmute::<_, cortex_m::Peripherals>(()).SCB }
}
pub const fn syst() -> SYST {
    unsafe { transmute::<_, cortex_m::Peripherals>(()).SYST }
}
pub const fn tpiu() -> TPIU {
    unsafe { transmute::<_, cortex_m::Peripherals>(()).TPIU }
}
