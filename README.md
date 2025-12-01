# Bot-a-Plot 
## This time in EGUI

<img src="./resources/images/ui_preview_basic.png" width="400px" />

Bot-a-Plot is a GCode generator/sender and post-processor that
turns SVG/PGF files into toolpaths for pen-plotting. It supports
GRBL dialects of GCode, and allows you to customize your post
processor depending on how your machine is configured. Virtual
and real toolchanges are supported for multiple pens.

Current features:

 * Import SVG and PGF files,
 * Post process geometry to tool-paths
 * Machine post editor
 * Selection/modification of existing geometry
    * Rotation
    * Translation
    * Scaling
    * Pen selection
 * Undo stack
 * Matting to paper/machine limits, and smart matting to optimize
   use of space
 * Spacemacs inspired command palette for quick triggering of
   menu commands
 * Basic machine control (shuttle/pen up/down/etc.)
 * Themeing to match your fancy WM customizations.

![Basic UI preview](./resources/images/ui_preview_basic.png)
![Basic UI preview](./resources/images/ui_preview_plot.png)
![Basic UI preview](./resources/images/ui_preview_spacecmd.png)
![Basic UI preview](./resources/images/ui_preview_theme.png)
