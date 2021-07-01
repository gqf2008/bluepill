# STM32F103C8T6 BLUE PILL

![BLUEPILL 图标](stm32f103c8t6-large.jpg "BLUEPILL")
### ST-LINK V2调试器

| ST-LINK | BLUE PILL |
| ------- | --------- |
| SWCLK   | CLK       |
| SWDIO   | DIO       |
| GND     | GND       |
| 3.3V    | 3.3V      |

### FT232 USB转串口

cargo run --release --example serial

| FT232   | BLUE PILL |
| ------- | --------- |
| 白线-RX | Tx-A9     |
| 蓝线-TX | RX-A10    |
| GND     | GND       |

### SSD1306 OLED

cargo run --release --example ssd1306

| SSD1306 | BLUE PILL |
| ------- | --------- |
| Vcc     | 3.3       |
| GND     | GND       |
| SCL     | PB8       |
| SDA     | PB9       |


### HR-SR04超声波模块

cargo run --release --example wave

| HR-SR04 | BLUE PILL |
| ------- | --------- |
| Vcc     | 5V        |
| Trig    | A0        |
| Echo    | A1        |
| GND     | GND       |

### MQ2烟雾传感器

cargo run --release --example mq2

| MQ2  | BLUE PILL |
| ---- | --------- |
| Vcc  | 3.3       |
| AOUT | A6        |
| DOUT | A7        |
| GND  | GND       |

### ESP8266-01S WIFI模块

cargo run --release --example wifi

![ESP8266-01S 图标](esp8266-01w.jpg "ESP8266-01S")
| ESP8266  | BLUEPILL |
| -------- | -------- |
| VCC      | 3.3      |
| CH_PD/EN | 3.3      |
| TX       | A3       |
| RX       | A2       |
| RST      |          |
| GND      | GND      |

### TM1637 4位数码管

cargo run --release --example tm1637

| TM1637 | BLUE PILL |
| ------ | --------- |
| CLK    | B6        |
| DIO    | B7        |
| VCC    | 3.3       |
| GND    | GND       |
| RST    |           |


### E-PAPER-7.5INCH-B电子墨水屏

![E-PAPER-7.5INCH-B 图标](e-paper-7.5inch-b.jpg "E-PAPER-7.5INCH-B")
TODO

| E-PAPER-7.5INCH-B | BLUE PILL |
| ----------------- | --------- |



