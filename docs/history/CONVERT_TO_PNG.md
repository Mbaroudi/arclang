# Convert ACC Diagram to PNG

## Option 1: Manual Screenshot (Recommended)

1. Open the HTML diagram:
   ```bash
   open adaptive_cruise_control.html
   ```

2. Take a screenshot of the full diagram (Cmd+Shift+4 on macOS)

3. Save as `docs/acc_diagram.png`

4. Update README.md line 258:
   ```markdown
   \![ACC System Architecture](docs/acc_diagram.png)
   ```

## Option 2: Using qlmanage (macOS)

```bash
# Convert HTML to PNG using Quick Look
qlmanage -t -s 2000 -o docs/ adaptive_cruise_control.html
mv docs/adaptive_cruise_control.html.png docs/acc_diagram.png
```

## Option 3: Install ImageMagick

```bash
# Install ImageMagick
brew install imagemagick

# Convert SVG to PNG
convert docs/adaptive_cruise_control_diagram.svg \
  -resize 1920x \
  docs/acc_diagram.png
```

## Option 4: Using Python with Selenium

```bash
pip install selenium pillow
python - << 'PYTHON'
from selenium import webdriver
from PIL import Image
import time

driver = webdriver.Chrome()
driver.get('file:///Users/malek/Arclang/adaptive_cruise_control.html')
time.sleep(2)
driver.save_screenshot('docs/acc_diagram.png')
driver.quit()
PYTHON
```

## Quick Test

After creating the PNG, verify it's readable:
```bash
ls -lh docs/acc_diagram.png
open docs/acc_diagram.png
```
