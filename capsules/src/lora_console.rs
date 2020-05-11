use core::cell::Cell;
use core::cmp;
use core::str;
use kernel::capabilities::LoraManagementCapability;
use kernel::common::cells::TakeCell;
use kernel::debug;

use kernel::hil::uart;
use kernel::hil::usb;

use kernel::introspection::KernelInfo;
use kernel::Kernel;
use kernel::ReturnCode;

// Since writes are character echoes, we do not need more than 4 bytes:
// the longest write is 3 bytes for a backspace (backspace, space, backspace).
pub static mut WRITE_BUF: [u8; 50] = [0; 50];
// Since reads are byte-by-byte, to properly echo what's typed,
// we can use a very small read buffer.
pub static mut READ_BUF: [u8; 25] = [0; 25];
// Commands can be up to 32 bytes long: since commands themselves are 4-5
// characters, limiting arguments to 25 bytes or so seems fine for now.
pub static mut COMMAND_BUF: [u8; 32] = [0; 32];

pub struct LoraConsole<'a, C: LoraManagementCapability> {
    uart: &'a dyn uart::UartData<'a>,
    tx_in_progress: Cell<bool>,
    tx_buffer: TakeCell<'static, [u8]>,
    rx_in_progress: Cell<bool>,
    rx_buffer: TakeCell<'static, [u8]>,
    command_buffer: TakeCell<'static, [u8]>,
    command_index: Cell<usize>,
    running: Cell<bool>,
    kernel: &'static Kernel,
    capability: C,
	command_count: Cell<usize>,
}

impl<'a, C: LoraManagementCapability> LoraConsole<'a, C> {
    pub fn new(
        uart: &'a dyn uart::UartData<'a>,
        tx_buffer: &'static mut [u8],
        rx_buffer: &'static mut [u8],
        cmd_buffer: &'static mut [u8],
        kernel: &'static Kernel,
        capability: C,
		
    ) -> LoraConsole<'a, C> {
        LoraConsole {
            uart: uart,
            tx_in_progress: Cell::new(false),
            tx_buffer: TakeCell::new(tx_buffer),
            rx_in_progress: Cell::new(false),
            rx_buffer: TakeCell::new(rx_buffer),
            command_buffer: TakeCell::new(cmd_buffer),
            command_index: Cell::new(0),
            running: Cell::new(false),
            kernel: kernel,
            capability: capability,
			command_count: Cell::new(0),
			
        }
    }

    pub fn start(&self) -> ReturnCode {
        if self.running.get() == false {
            self.rx_buffer.take().map(|buffer| {
                self.rx_in_progress.set(true);
                self.uart.receive_buffer(buffer, 1);
                self.running.set(true);
                //debug!("Starting LoRa console");
            });}
			let command = "AT?";
			self.tx_buffer.take().map(|buffer| {
				let len = cmp::min(command.len(), buffer.len());
				let mut i=0;
				for c in command.chars() {
					buffer[i] = c as u8;
					i=i+1;
				}
				self.uart.transmit_buffer(buffer, len);
				let comm = 0;
				self.command_count.set(comm+1);
		});
        ReturnCode::SUCCESS
		}
	
    // Process the command in the command buffer and clear the buffer.
    fn read_command(&self) {
        self.command_buffer.map(|command| {
            let mut terminator = 0;
            let len = command.len();
            for i in 0..len {
                if command[i] == 0 {
                    terminator = i;
                    break;
                }
            }
            //debug!("Command: {}-{} {:?}", start, terminator, command);
            // A command is valid only if it starts inside the buffer,
            // ends before the beginning of the buffer, and ends after
            // it starts.
            if terminator > 0 {
                let cmd_str = str::from_utf8(&command[0..terminator]);
				
                match cmd_str {
                    Ok(s) => {
                        let clean_str = s.trim();
						let command;
						if clean_str.trim() =="OK" && self.command_count.get()==1{
							//debug!("Inside Join");
							command = "AT+JOIN?";
							self.tx_buffer.take().map(|buffer| {
								let len = cmp::min(command.len(), buffer.len());
								let mut i=0;
								for c in command.chars() {
									buffer[i] = c as u8;
									i=i+1;
								}
								self.uart.transmit_buffer(buffer, len);
								let comm = self.command_count.get();
								self.command_count.set(comm+1);
							});
                        }
						else if clean_str.trim() =="OK" && self.command_count.get()==2{
							//debug!("Inside Network");
							command = "AT+NJS?";
							self.tx_buffer.take().map(|buffer| {
								let len = cmp::min(command.len(), buffer.len());
								let mut i=0;
								for c in command.chars() {
									buffer[i] = c as u8;
									i=i+1;
								}
								self.uart.transmit_buffer(buffer, len);
								let comm = self.command_count.get();
								self.command_count.set(comm+1);
							});
						}
						else if self.command_count.get()==3 {
							//debug!("Further inside Netowk ");
							if clean_str.trim() =="0"{
								//debug!("Netowk is not Joined");
								command = "AT+NJS?";
							}
							else if clean_str.trim() =="OK"{
								//debug!("Netowk is conecting........");
								command = "AT+NJS?";
								self.tx_buffer.take().map(|buffer| {
								let len = cmp::min(command.len(), buffer.len());
								let mut i=0;
								for c in command.chars() {
									buffer[i] = c as u8;
									i=i+1;
								}
								self.uart.transmit_buffer(buffer, len);
								let comm = self.command_count.get();
								self.command_count.set(comm+1);
								});
							}
						}
						else if self.command_count.get()==4 && clean_str.trim() =="1"{
							//debug!("Inside send data");
							command = "AT+SEND=50:Hello World";
								self.tx_buffer.take().map(|buffer| {
								let len = cmp::min(command.len(), buffer.len());
								let mut i=0;
								for c in command.chars() {
									buffer[i] = c as u8;
									i=i+1;
								}
								self.uart.transmit_buffer(buffer, len);
								let comm = self.command_count.get();
								self.command_count.set(comm+1);
								});
						}
						else if self.command_count.get()==5 && clean_str.trim() =="OK"{
							debug!("Data sent Successfully!");
						}
						else {
                            debug!("Restart everything!\n");
							let status = "AT?";
							self.tx_buffer.take().map(|buffer| {
							let len = cmp::min(status.len(), buffer.len());
							let mut i=0;
							for c in status.chars() {
								buffer[i] = c as u8;
								i=i+1;
								}
							self.uart.transmit_buffer(buffer, len);
							let comm = 0;
							self.command_count.set(comm+1);
							});
                        }
                    }
                    Err(_e) => debug!("Invalid command: {:?}", command),
                };
            }
        });
        self.command_buffer.map(|command| {
            command[0] = 0;
        });
        self.command_index.set(0);
    }
	
	/*

    fn write_byte(&self, byte: u8) -> ReturnCode {
        if self.tx_in_progress.get() {
            ReturnCode::EBUSY
        } else {
            self.tx_in_progress.set(true);
            self.tx_buffer.take().map(|buffer| {
                buffer[0] = byte;
				debug!("Tx buffer start!!");
                self.uart.transmit_buffer(buffer, 1);
            });
            ReturnCode::SUCCESS
        }
    }

    fn write_bytes(&self, bytes: &[u8]) -> ReturnCode {
        if self.tx_in_progress.get() {
            ReturnCode::EBUSY
        } else {
            self.tx_in_progress.set(true);
            self.tx_buffer.take().map(|buffer| {
                let len = cmp::min(bytes.len(), buffer.len());
                for i in 0..len {
                    buffer[i] = bytes[i];
                }
				//debug!("Tx buffer start!!");
                self.uart.transmit_buffer(buffer, len);
            });
            ReturnCode::SUCCESS
        }
    }*/
}


impl<'a, C: LoraManagementCapability> uart::TransmitClient for LoraConsole<'a, C> {
    fn transmitted_buffer(&self, buffer: &'static mut [u8], _tx_len: usize, _rcode: ReturnCode) {
        // Either print more from the AppSlice or send a callback to the
        // application.
		//debug!("Tx buffer return!!");
        self.tx_buffer.replace(buffer);
        self.tx_in_progress.set(false);
    }
}

impl<'a, C: LoraManagementCapability> uart::ReceiveClient for LoraConsole<'a, C> {
    fn received_buffer(
        &self,
        read_buf: &'static mut [u8],
        rx_len: usize,
        _rcode: ReturnCode,
        error: uart::Error,
    ) {
        let mut execute = false;
        if error == uart::Error::None {
            match rx_len {
                0 => debug!("LoraConsole had read of 0 bytes"),
                1 => {
                    self.command_buffer.map(|command| {
                        let index = self.command_index.get() as usize;
                        if read_buf[0] == ('\n' as u8) || read_buf[0] == ('\r' as u8) {
                            //debug!("****************At 1st level");
							execute = true;
                            //self.write_bytes(&['\r' as u8, '\n' as u8]);
							
                        } else if read_buf[0] == ('\x08' as u8) && index > 0 {
							//debug!("****************At 2nd level");
                            // Backspace, echo and remove last byte
                            // Note echo is '\b \b' to erase
                            //self.write_bytes(&['\x08' as u8, ' ' as u8, '\x08' as u8]);
                            command[index - 1] = '\0' as u8;
                            self.command_index.set(index - 1);
                        } else if index < (command.len() - 1) && read_buf[0] < 128 {
							//debug!("****************At 3rd level");
                            // For some reason, sometimes reads return > 127 but no error,
                            // which causes utf-8 decoding failure, so check byte is < 128. -pal

                            // Echo the byte and store it
                            //self.write_byte(read_buf[0]);
							
                            command[index] = read_buf[0];
                            self.command_index.set(index + 1);
                            command[index + 1] = 0;
                        }
                    });
                }
                _ => debug!(
                    "LoraConsole issues reads of 1 byte, but receive_complete was length {}",
                    rx_len
                ),
            };
        }
        self.rx_in_progress.set(true);
        self.uart.receive_buffer(read_buf, 1);
		//debug!("Input to NRf detected!!");
        if execute {
            self.read_command();
        }
    }
}
