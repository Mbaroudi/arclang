# Generate Screenshot Instructions

To add the ACC diagram screenshot to README:

1. Open the generated HTML diagram:
   ```bash
   open adaptive_cruise_control.html
   ```

2. Take a screenshot of the diagram showing:
   - All 10 components
   - Connection arrows
   - Professional styling
   
3. Save as `docs/acc_diagram_screenshot.png`

4. Update README.md line 258:
   ```markdown
   \![ACC System Architecture](docs/acc_diagram_screenshot.png)
   ```

Alternatively, use the HTML file directly or convert to SVG:
   ```bash
   # If you have wkhtmltoimage installed
   wkhtmltoimage adaptive_cruise_control.html docs/acc_diagram.png
   ```
