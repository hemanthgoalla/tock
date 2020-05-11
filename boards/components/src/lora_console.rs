
use capsules::lora_console;
use capsules::virtual_uart::{MuxUart, UartDevice};
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::static_init;

pub struct LoraConsoleComponent {
    board_kernel: &'static kernel::Kernel,
    uart_mux: &'static MuxUart<'static>,
}

impl LoraConsoleComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        uart_mux: &'static MuxUart,
    ) -> LoraConsoleComponent {
        LoraConsoleComponent {
            board_kernel: board_kernel,
            uart_mux: uart_mux,
        }
    }
}

pub struct Capability;
unsafe impl capabilities::LoraManagementCapability for Capability {}

impl Component for LoraConsoleComponent {
    type StaticInput = ();
    type Output = &'static lora_console::LoraConsole<'static, Capability>;

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        // Create virtual device for console.
        let console_uart = static_init!(UartDevice, UartDevice::new(self.uart_mux, true));
        console_uart.setup();

        let console = static_init!(
            lora_console::LoraConsole<'static, Capability>,
            lora_console::LoraConsole::new(
                console_uart,
                &mut lora_console::WRITE_BUF,
                &mut lora_console::READ_BUF,
                &mut lora_console::COMMAND_BUF,
                self.board_kernel,
                Capability,
            )
        );
        hil::uart::Transmit::set_transmit_client(console_uart, console);
        hil::uart::Receive::set_receive_client(console_uart, console);

        console
    }
}
