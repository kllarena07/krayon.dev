# Making My Own Terminal (pt 1/n)
_This project was a mistake_

**DISCLAIMER:** this blog post will cover terminals for macOS/Linux kernels, and not Windows! Windows terminals may work differently than what I will cover here.

## 🎯 The Why

Back in the summer I did a little bit of growth work for a startup called [Warp](https://www.warp.dev/). Eventually I decided to quit this work since I wanted to focus more on becoming a better engineer. In hindsight, it’s safe to say that I wasn’t really that good at the growth stuff since I had only garnered 40k total views in the span of 3 months. If for whatever reason you want a more comprehensive detail of my work, you can view all the stats [here](https://docs.google.com/spreadsheets/d/1GsuNtnZ7Y_NDgOqKT5SkC2zSlXGzskj0K54nP3cdUz0/edit?usp=sharing).

During that time, I began to wonder _actually_ how a terminal works. It was just until now, December 2025, that I decided to take action and dive a little deeper.

## ⚙️ The How

My initial research first lead me to [this](https://dev.to/therubberduckiee/demystifying-the-terminal-how-it-works-behind-the-scenes-50h6) article, funnily enough by Warp.

While the article does a decent job at summarizing how everything works at a high-level, it does just that: a high-level explanation. So, I wanted to take the time to write a blog post about the lower level intricacies about the full stack of a terminal app.

To help understand what is going on when you use your terminal, let’s use this diagram:
```
       USER
    (Keyboard)
         │
         ▼
┌───────────────────┐
│ Terminal Emulator │ (e.g., iTerm2, Alacritty, GNOME Terminal)
│   (The App)       │
└────────┬──────────┘
         │
         │ writes to /dev/ptmx (Master)
         ▼
┌───────────────────┐
│    PTY Master     │ ◄──┐
└───────────────────┘    │
         │               │  KERNEL SPACE
  Line Discipline        │  (Handles Ctrl+C, Backspace, 
         │               │   Echoing, Line Buffering)
         ▼               │
┌───────────────────┐    │
│    PTY Slave      │ ◄──┘
│   (/dev/pts/N)    │
└────────┬──────────┘
         │
         │ stdin / stdout
         ▼
┌───────────────────┐
│      Shell        │ (e.g., bash, zsh, fish)
│    (Process)      │
└───────────────────┘
```

Here, the data travels DOWN to the shell process and then back UP to the user.

With this in mind, let’s get started explaining the diagram.


The terminal that you typically use and see is short “terminal emulator”, which is just a software imitation (or an emulation) of a [PHYSICAL terminal](https://en.wikipedia.org/wiki/Computer_terminal) (otherwise called a TTY), which people used to use in the early days of computers. However, do note that this terminal emulator only emulates the screen and keyboard of the physical terminal. If you come from a web development background, you can think of this app as just the “frontend”. This is an important nuance when trying to understand the next component: the PTY.

The PTY is a software driver that is a pair of character devices (I’ll get to what those are in a bit) split into a Master PTY and a Slave PTY. A character device is a device file (an interface to a device driver that appears in the file system as if it were an ordinary file) that allows user space applications to communicate with hardware (or a kernel service) as if it were a stream of bytes.


On user input submission, the terminal emulator writes to the PTY Master via a [file descriptor](https://en.wikipedia.org/wiki/File_descriptor). The master side then uses a Line Discipline to communicate with the PTY Slave. The Line Discipline is a kernel module that holds data for what was actually typed in a buffer (which is in kernel space since it’s a kernel module).


**Let’s stop for a second though since this part is my favorite and rewind.**


Back when we used actual terminals, the flow of the data from your keyboard to the terminal would go:
```
   [ USER ] (Physical interaction)
       |
       v
+-----------------------------------------------------------+
|  PHYSICAL TERMINAL (The Hardware)                         |
|  - Keyboard: Converts keypress to serial pulses           |
|  - Printer/Screen: Converts serial pulses to characters   |
+-----------------------------------------------------------+
       |
   [ SERIAL CABLE / RS-232 WIRE ]
       |
       v
+-----------------------------------------------------------+
|  UART DRIVER (Kernel Space - Hardware Manager)            |
|  - Manages the physical "Serial Port" chips               |
|  - Translates electrical pulses <--> raw bytes            |
+-----------------------------------------------------------+
       |
       v
+-----------------------------------------------------------+
|  LINE DISCIPLINE (Kernel Space - The Logic Module)        |
|  - [ INTERNAL BUFFER ]: Holds typed characters in RAM     |
+-----------------------------------------------------------+
       |
       v
+-----------------------------------------------------------+
|  CHARACTER DEVICE INTERFACE (e.g., /dev/ttyS0)            |
|  - The "File" entry point that connects Kernel to User    |
|  - Provides the API: open(), read(), write(), ioctl()     |
+-----------------------------------------------------------+
       |
       v
+-----------------------------------------------------------+
|  THE SHELL                                                |
|  - Calls read(): "Give me a line of text"                 |
|  - Calls write(): "Send this output to the user"          |
+-----------------------------------------------------------+
```

Recall from a few seconds ago:
> _A character device is a device file (an interface to a device driver that appears in the file system as if it were an ordinary file) that allows user space applications to communicate with hardware (or a kernel service) as if it were a stream of bytes._

Additionally:
> _The Line Discipline is a kernel module that holds data for what was actually typed in a buffer (which is in kernel space since it’s a kernel module)._

Since the PTY Master is a character device, this whole step in the PTY process actually deceives the Line Discipline into thinking it’s a UART driver. This allows for interoperability of your terminal emulator. Pretty cool right?

Anyways, back to our regularly scheduled program.
```
       USER
    (Keyboard)
         │
         ▼
┌───────────────────┐
│ Terminal Emulator │ (e.g., iTerm2, Alacritty, GNOME Terminal)
│   (The App)       │
└────────┬──────────┘
         │
         │ writes to /dev/ptmx (Master)
         ▼
┌───────────────────┐
│    PTY Master     │ ◄──┐
└───────────────────┘    │
         │               │  KERNEL SPACE
  Line Discipline        │  (Handles Ctrl+C, Backspace, 
         │               │   Echoing, Line Buffering)
         ▼               │
┌───────────────────┐    │
│    PTY Slave      │ ◄──┘
│   (/dev/pts/N)    │
└────────┬──────────┘
         │
         │ stdin / stdout
         ▼
┌───────────────────┐
│      Shell        │ (e.g., bash, zsh, fish)
│    (Process)      │
└───────────────────┘
```

To make the next part easier to understand, the shell always calls `read()` when it can. However, data from the PTY Master to the PTY Slave via the Line Discipline is not sent immediately. Instead, the Line Discipline holds it in RAM in the kernel space until a `\n` character is received. 


So, to explain how the terminal emulator input gets to the shell, the shell calls `read` to read input and then the PTY Slave forwards the data to the shell process when permitted by the Line Discipline.

Once the shell runs the command, the output is sent back up and rendered to your screen.

## 🏁 The “Complete” Terminal

Here’s what you came for, a demo of the terminal I made:

<video width="640" controls>
  <source src="demo.mp4" type="video/mp4">
  Your browser does not support the video tag.
</video>

While it’s not the prettiest, it works.

Funnily enough, the actual code for the terminal isn’t as complex as you would think. But this is probably due to the fact that I’ve used crates to abstract away some of the finer grain details. View the code [here](https://github.com/kllarena07/kterm).

This is only part 1/n because I have future plans to improve it, though I'm not sure how long it will take:

1. Write my own TUI since I used a GUI one (`egui`)
2. Switch to use a different pty crate since working with `portable_pty` results in some inconsistent output (e.g. the shell command output wouldn’t be the only thing coming back)

I am anticipating that writing my own emulator will be a lot more work with dealing how to render graphics to the computer which is not my expertises, hence the “this project was a mistake” subtitle.

I’m looking forward to the challenge though! So, stay tuned.

If I got anything wrong, feel free to correct me by DMing me on Twitter.

_Written with ❤️ by Krayon_  
Follow me: [x.com/krayon](https://x.com/krayondev)

