**Product Requirements Document (PRD) for RepoDiff UI (PyQt)**

### 1. Overview
This PRD outlines the requirements for a **modern, dark-mode enabled UI** for RepoDiff, built using **PyQt**. The UI should provide an intuitive interface for users to configure diff settings, select commits, and generate a structured diff file for LLM consumption.

### 2. Key Objectives
1. **Provide an intuitive UI** for configuring file filters, diff settings, and context rules.
2. **Support dark mode** with modern aesthetics.
3. **Enable commit selection** from a dropdown or interactive selection.
4. **Allow easy export** of the processed diff to a file.
5. **Ensure responsiveness** and usability across different screen sizes.
6. **Keep UI simple** without embedding a diff viewer, as users will edit the output in their own text editor.

### 3. UI Design
#### **3.1. Main Window Components**
1. **Commit Selection Panel**
   - Dropdowns to select `commit1` and `commit2`.
2. **Configuration Panel**
   - File pattern selector (`*.cs`, `*.py`, etc.).
   - Toggle for `include_entire_file_with_signatures`.
   - Input field for `method_body_threshold`.
   - Context lines selection (`-U` value).
3. **Export Options**
   - Output path.
   - “Pack” button to trigger diff generation and export output as `.md` file.
4. **Dark Mode Toggle**
   - Integrated style switcher for light/dark themes.

### 4. Dark Mode Implementation
- Use **Qt’s Fusion theme** with a dark stylesheet.
- Custom stylesheets for buttons, dropdowns, and text areas to ensure consistency.
- Automatic detection of system theme (if possible).

### 5. Interaction Flow
1. **User selects two commits** from dropdowns.
2. **User configures filters and options** (e.g., method body removal, context lines).
3. **User clicks “Pack”** to generate the diff file.
4. **File is automatically saved/exported**, ready for upload to an LLM.

### 6. Sample Pseudocode
```python
from PyQt6.QtWidgets import QApplication, QMainWindow, QPushButton, QVBoxLayout, QFileDialog
from PyQt6.QtGui import QPalette, QColor

class RepoDiffUI(QMainWindow):
    def __init__(self):
        super().__init__()
        self.initUI()

    def initUI(self):
        self.setWindowTitle("RepoDiff")
        self.setGeometry(100, 100, 600, 400)

        # Dark mode setup
        palette = QPalette()
        palette.setColor(QPalette.ColorRole.Window, QColor(53, 53, 53))
        palette.setColor(QPalette.ColorRole.WindowText, QColor(255, 255, 255))
        self.setPalette(palette)

        # Main Layout
        layout = QVBoxLayout()

        # Pack Button
        self.pack_button = QPushButton("Pack")
        self.pack_button.clicked.connect(self.generate_diff)
        layout.addWidget(self.pack_button)

        self.setLayout(layout)

    def generate_diff(self):
        file_path, _ = QFileDialog.getSaveFileName(self, "Save Diff File", "", "Markdown Files (*.md);;All Files (*)")
        if file_path:
            with open(file_path, 'w') as f:
                f.write("Generated diff output...")

app = QApplication([])
window = RepoDiffUI()
window.show()
app.exec()
```

### 7. Expected Outcome
- A modern UI with a **clean, structured layout**.
- **Dark mode support** with clear text visibility.
- **No embedded diff viewer**, only file export.
- **Seamless interaction** for configuring and generating diffs.
- **Optimized for LLMs and manual editing.**

### 8. Summary
RepoDiff’s UI will be developed in **PyQt**, providing a **modern, user-friendly interface** with dark mode support. The UI will enable easy configuration, commit selection, and diff generation, making RepoDiff an accessible and efficient tool for code review workflows.

