# sliders

sliders in the terminal

# screenshot

![screenshot](screenshot.png)

# example 

For this example, let's store counter in 2 files

```bash
$ echo 0 > hello.counter
$ echo 0 > world.counter
```

Then, let's launch two sliders which will write/read to/from those files:

```bash
cargo run -- --name hello --get 'cat hello.counter' --set 'echo % > hello.counter' \
             --name world --get 'cat world.counter' --set 'echo % > world.counter'
``` 

# backlight-mixer

You can use this to update your backlight with [backlight-mixer](https://github.com/yazgoo/backlight-mixer).

cargo run -- --name backlight --get 'backlight-mixer' --set 'backlight-mixer %'
