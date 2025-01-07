#[derive(Debug, Clone, Copy)]
pub enum Button {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right
}

#[derive(Debug, Default)]
pub struct ButtonStates {
    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

#[derive(Debug)]
pub struct Controller {
    shift_register: u8,
    buttons: ButtonStates,
    strobe: bool,
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            shift_register: 0,
            buttons: ButtonStates::default(),
            strobe: false,
        }
    }

    /// Write to $4016 to control controller strobe
    /// bit 0: strobe bit (1 = poll input, 0 = read mode)
    pub fn write(&mut self, value: u8) {
        let new_strobe = value & 0x01 == 0x01;
        
        // If strobe transitions from 0->1, latch current button states
        if !self.strobe && new_strobe {
            self.latch_buttons();
        }
        
        self.strobe = new_strobe;
    }


    pub fn read(&mut self) -> u8 {

        if self.strobe {
            return if self.shift_register & 0x80 != 0 { 1 } else { 0 };
        }

        let result = if self.shift_register & 0x80 != 0 { 1 } else { 0 };
        self.shift_register <<= 1;
        
        result
    }

    /// Update current button states from input device
    pub fn set_button_states(&mut self, states: ButtonStates) {
        self.buttons = states;
        
        // If currently strobing, immediately latch new states
        if self.strobe {
            self.latch_buttons();
        }
    }

    /// Internal function to latch current button states into shift register
    fn latch_buttons(&mut self) {
        self.shift_register = 0;
        self.shift_register |= if self.buttons.a      { 0x80 } else { 0 };
        self.shift_register |= if self.buttons.b      { 0x40 } else { 0 };
        self.shift_register |= if self.buttons.select { 0x20 } else { 0 };
        self.shift_register |= if self.buttons.start  { 0x10 } else { 0 };
        self.shift_register |= if self.buttons.up     { 0x08 } else { 0 };
        self.shift_register |= if self.buttons.down   { 0x04 } else { 0 };
        self.shift_register |= if self.buttons.left   { 0x02 } else { 0 };
        self.shift_register |= if self.buttons.right  { 0x01 } else { 0 };
    }

    pub fn set_button(&mut self, button: Button, pressed: bool) {
        match button {
            Button::A => self.buttons.a = pressed,
            Button::B => self.buttons.b = pressed,
            Button::Select => self.buttons.select = pressed,
            Button::Start => self.buttons.start = pressed,
            Button::Up => self.buttons.up = pressed,
            Button::Down => self.buttons.down = pressed,
            Button::Left => self.buttons.left = pressed,
            Button::Right => self.buttons.right = pressed,
        }

        // If currently strobing, immediately latch new states
        if self.strobe {
            self.latch_buttons();
        }
    }
}