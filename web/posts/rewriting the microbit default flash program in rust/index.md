# Rewriting the Microbit V2 Default Flash Program in Rust
_Just the beginning of my embedded Rust journey_

When I first got my Microbit V2 it came with this fun program where you could click the “A” and “B” buttons on the back and it would display either a happy/sad face accompanied with a happy/sad sound depending on which button you clicked.

I dismissed this quickly because I thought it was goofy and replace the firmware with a [“Hello World” Blink LED program in Rust](https://decorous-forgery-74e.notion.site/Blink-in-Rust-282b898d9b9e80529689ef8835355207?pvs=74). But once that was done I began to miss the program. So I restored it myself, and learned a lot in doing so.

Just like the blog post about linked in above, I rewrote this program in Rust while staying pretty close to the metal (very non-Rustic I know but be quiet because it was fun).

**Here’s the code for it:**
```Rust
#![no_std]
#![no_main]

use core::ptr::{read_volatile, write_volatile};

use cortex_m::asm::nop;
use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    const GPIO0_PINCNF0_SPEAKER_ADDR: *mut u32 = 0x5000_0700 as *mut u32;
    const GPIO0_PINCNF14_BTN_A_ADDR: *mut u32 = 0x5000_0738 as *mut u32;
    const GPIO0_PINCNF23_BTN_B_ADDR: *mut u32 = 0x5000_075C as *mut u32;
    const DIR_OUTPUT_POS: u32 = 0;

    const ROWS: [*mut u32; 5] = [
        0x5000_0754 as *mut u32, // ROW 1
        0x5000_0758 as *mut u32, // ROW 2
        0x5000_073C as *mut u32, // ROW 3
        0x5000_0760 as *mut u32, // ROW 4
        0x5000_074C as *mut u32, // ROW 5
    ];

    const ROW_PIN_NUMBS: [u32; 5] = [
        21, // ROW 1
        22, // ROW 2
        15, // ROW 3
        24, // ROW 4
        19, // ROW 5
    ];

    const COLUMNS: [*mut u32; 5] = [
        0x5000_0770 as *mut u32, // COL 1
        0x5000_072C as *mut u32, // COL 2
        0x5000_077C as *mut u32, // COL 3
        0x5000_0A14 as *mut u32, // COL 4, P1 (0x50000300); PIN 5 (0x714); 0x50000300 + 0x714
        0x5000_0778 as *mut u32, // COL 5
    ];

    unsafe {
        // configure BTN_B (Port 0, Pin 14)
        write_volatile(GPIO0_PINCNF14_BTN_A_ADDR, 0 << DIR_OUTPUT_POS);
        // configure BTN_B (Port 0, Pin 23)
        write_volatile(GPIO0_PINCNF23_BTN_B_ADDR, 0 << DIR_OUTPUT_POS);

        write_volatile(GPIO0_PINCNF0_SPEAKER_ADDR, 1 << DIR_OUTPUT_POS);
        write_volatile(ROWS[1], 1 << DIR_OUTPUT_POS); // row 2
        write_volatile(ROWS[3], 1 << DIR_OUTPUT_POS); // row 4
        write_volatile(ROWS[4], 1 << DIR_OUTPUT_POS); // row 5
    }

    rtt_init_print!();
    rprintln!("Starting...");

    const MULTIPLEX_DELAY: u32 = 1_000;
    const GPIO0_OUT_ADDR: *mut u32 = 0x5000_0504 as *mut u32;
    const GPIO0_OUTSET_ADDR: *mut u32 = 0x5000_0508 as *mut u32;
    const GPIO0_OUTCLR_ADDR: *mut u32 = 0x5000_050C as *mut u32;
    const GPIO0_IN_ADDR: *mut u32 = 0x5000_0510 as *mut u32;
    const GPIO0_IN_BTN_A_POS: u32 = 14;
    const GPIO0_IN_BTN_B_POS: u32 = 23;
    const GPIO0_OUT_SPEAKER_POS: u32 = 0;

    fn set_columns(activation_matrix: &[u32; 5]) {
        for i in 0..activation_matrix.len() {
            unsafe {
                write_volatile(COLUMNS[i], activation_matrix[i] << DIR_OUTPUT_POS);
            }
        }
    }

    fn draw_eyes() {
        unsafe {
            write_volatile(GPIO0_OUTSET_ADDR, 1 << ROW_PIN_NUMBS[1]);
            write_volatile(GPIO0_OUTCLR_ADDR, 1 << ROW_PIN_NUMBS[3]);
            write_volatile(GPIO0_OUTCLR_ADDR, 1 << ROW_PIN_NUMBS[4]);
        }
        const ACTIVATED: [u32; 5] = [0, 1, 0, 1, 0];
        set_columns(&ACTIVATED);
    }

    fn draw_smile_cheeks() {
        unsafe {
            // Drawing the "cheeks"
            write_volatile(GPIO0_OUTCLR_ADDR, 1 << ROW_PIN_NUMBS[1]);
            write_volatile(GPIO0_OUTSET_ADDR, 1 << ROW_PIN_NUMBS[3]);
            write_volatile(GPIO0_OUTCLR_ADDR, 1 << ROW_PIN_NUMBS[4]);
        }
        const ACTIVATED: [u32; 5] = [1, 0, 0, 0, 1];
        set_columns(&ACTIVATED);
    }

    fn draw_smile_lips() {
        unsafe {
            // Drawing row 5
            write_volatile(GPIO0_OUTCLR_ADDR, 1 << ROW_PIN_NUMBS[1]);
            write_volatile(GPIO0_OUTCLR_ADDR, 1 << ROW_PIN_NUMBS[3]);
            write_volatile(GPIO0_OUTSET_ADDR, 1 << ROW_PIN_NUMBS[4]);
        }
        const ACTIVATED: [u32; 5] = [0, 1, 1, 1, 0];
        set_columns(&ACTIVATED);
    }

    fn smile() {
        draw_eyes();

        for _ in 0..MULTIPLEX_DELAY {
            nop();
        }

        draw_smile_cheeks();

        for _ in 0..MULTIPLEX_DELAY {
            nop();
        }

        draw_smile_lips();

        for _ in 0..MULTIPLEX_DELAY {
            nop();
        }
    }

    fn draw_frown_lips() {
        unsafe {
            // Drawing row 5
            write_volatile(GPIO0_OUTCLR_ADDR, 1 << ROW_PIN_NUMBS[1]);
            write_volatile(GPIO0_OUTSET_ADDR, 1 << ROW_PIN_NUMBS[3]);
            write_volatile(GPIO0_OUTCLR_ADDR, 1 << ROW_PIN_NUMBS[4]);
        }

        const ACTIVATED: [u32; 5] = [0, 1, 1, 1, 0];
        set_columns(&ACTIVATED);
    }

    fn draw_frown_cheeks() {
        unsafe {
            // Drawing the "cheeks"
            write_volatile(GPIO0_OUTCLR_ADDR, 1 << ROW_PIN_NUMBS[1]);
            write_volatile(GPIO0_OUTCLR_ADDR, 1 << ROW_PIN_NUMBS[3]);
            write_volatile(GPIO0_OUTSET_ADDR, 1 << ROW_PIN_NUMBS[4]);
        }
        const ACTIVATED: [u32; 5] = [1, 0, 0, 0, 1];
        set_columns(&ACTIVATED);
    }

    fn frown() {
        draw_eyes();

        for _ in 0..MULTIPLEX_DELAY {
            nop();
        }

        draw_frown_cheeks();

        for _ in 0..MULTIPLEX_DELAY {
            nop();
        }

        draw_frown_lips();

        for _ in 0..MULTIPLEX_DELAY {
            nop();
        }
    }

    let mut is_smiling = true;

    fn wind_up() {
        smile();
        let mut delay = 400;
        while delay > 0 {
            unsafe {
                write_volatile(GPIO0_OUT_ADDR, 1 << GPIO0_OUT_SPEAKER_POS);
            }

            draw_eyes();
            for _ in 0..delay {
                nop();
            }

            draw_smile_cheeks();
            for _ in 0..delay {
                nop();
            }

            unsafe {
                write_volatile(GPIO0_OUT_ADDR, 0 << GPIO0_OUT_SPEAKER_POS);
            }

            draw_smile_lips();

            for _ in 0..delay {
                nop();
            }
            delay -= 1;
        }
    }

    fn wind_down() {
        let mut delay = 0;
        unsafe {
            while delay < 400 {
                write_volatile(GPIO0_OUT_ADDR, 1 << GPIO0_OUT_SPEAKER_POS);

                draw_eyes();
                for _ in 0..delay {
                    nop();
                }
                draw_frown_cheeks();
                for _ in 0..delay {
                    nop();
                }
                write_volatile(GPIO0_OUT_ADDR, 0 << GPIO0_OUT_SPEAKER_POS);

                draw_frown_lips();
                for _ in 0..delay {
                    nop();
                }
                delay += 1;
            }
        }
    }

    loop {
        unsafe {
            let input_port_val: u32 = read_volatile(GPIO0_IN_ADDR);
            let gpio0_in_btn_a_val: u32 = (input_port_val >> GPIO0_IN_BTN_A_POS) & 1;
            let gpio0_in_btn_b_val: u32 = (input_port_val >> GPIO0_IN_BTN_B_POS) & 1;

            rprintln!("{}, {}", gpio0_in_btn_a_val, gpio0_in_btn_b_val);

            if gpio0_in_btn_a_val == 0 && gpio0_in_btn_b_val == 0 {
                nop();
            } else if gpio0_in_btn_a_val == 0 && gpio0_in_btn_b_val == 1 {
                wind_up();
                is_smiling = true;
            } else if gpio0_in_btn_a_val == 1 && gpio0_in_btn_b_val == 0 {
                wind_down();
                is_smiling = false;
            }

            if is_smiling {
                smile();
            } else {
                frown();
            }
        }
    }
}
```
I know it’s not pretty, but it gets the job done and I see no reason to optimize it. The logic of the code isn’t that complicated is the usual. Configure the pins to either be output/input, write output the registers and read the input from the registers. The only “complicated” thing here is the timing, which actually can be improved since I can see some of the LEDs being cleared.

Notice how there is drawing logic in the wind_up() and wind_down() function. That’s because calling those functions are actually blocking (which makes sense) and the logic near the end of the loop doesn’t execute. So, we have to call the drawing logic in side of those functions while also still adhering to the timing of the speakers (since outputting sound on a speaker is just driving the pin state of the speaker HIGH and LOW in a quick successive manner).

And that’s it!

**Here’s the Program in Action:**
<video width="640" height="360" controls>
  <source src="./IMG_7392.mov" type="video/mp4">
  Your browser does not support the video tag.
</video>

_Written with ❤️ by Krayon_  
Follow me: [x.com/krayon](https://x.com/krayondev)

