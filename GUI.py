import os
import sys
import tkinter as tk
import tkinter.font as tkfont
import numpy as np
from PIL import Image, ImageDraw, ImageTk
import matplotlib.pyplot as plt
from matplotlib.figure import Figure
from matplotlib.backends.backend_agg import FigureCanvasAgg
import matplotlib.font_manager as fm

# Enable Windows DPI Awareness for TV/Monitor clarity
if sys.platform.startswith("win"):
    try:
        import ctypes
        ctypes.windll.shcore.SetProcessDpiAwareness(1)
    except Exception:
        try:
            ctypes.windll.user32.SetProcessDPIAware()
        except Exception:
            pass


def load_and_register_font():
    """
    Registers the local .otf / .ttf font file with both Windows GDI (for Tkinter)
    and FreeType (for Matplotlib) without requiring admin system install.
    """
    search_dirs = [
        ".",
        "fonts",
        "C:/Windows/Fonts",
        os.path.expandvars("%LOCALAPPDATA%/Microsoft/Windows/Fonts"),
    ]
    font_name = "Segoe UI"
    for folder in search_dirs:
        if os.path.exists(folder):
            try:
                for file in os.listdir(folder):
                    if "vipna" in file.lower() and file.lower().endswith((".ttf", ".otf")):
                        full_path = os.path.abspath(os.path.join(folder, file))
                        
                        # 1. Register with Windows GDI so Tkinter recognizes it immediately
                        if sys.platform.startswith("win"):
                            try:
                                import ctypes
                                FR_PRIVATE = 0x10
                                ctypes.windll.gdi32.AddFontResourceExW(full_path, FR_PRIVATE, 0)
                            except Exception:
                                pass

                        # 2. Register with Matplotlib FontManager
                        try:
                            fm.fontManager.addfont(full_path)
                            prop = fm.FontProperties(fname=full_path)
                            font_name = prop.get_name()
                            return font_name
                        except Exception:
                            return "Vipnagorgialla"
            except Exception:
                continue
    return font_name


ACTIVE_FONT = load_and_register_font()

# Configure Matplotlib fonts
plt.rcParams['font.sans-serif'] = [ACTIVE_FONT, 'Segoe UI', 'DejaVu Sans', 'Arial']
plt.rcParams['font.family'] = 'sans-serif'


def safe_rounded_rectangle(draw, bbox, radius, fill, outline=None, width=1):
    x0, y0, x1, y1 = bbox
    if x1 <= x0 or y1 <= y0:
        return
    r = min(radius, max(1, int((x1 - x0) // 2)), max(1, int((y1 - y0) // 2)))
    draw.rounded_rectangle([x0, y0, x1, y1], radius=r, fill=fill, outline=outline, width=width)


class ScrollableGlassCalculatorApp:
    def __init__(self, root):
        self.root = root
        self.root.title("Calculadora 3D & Shopee")
        self.root.geometry("980x680")
        self.root.minsize(820, 520)

        self.font_family = ACTIVE_FONT
        self.scale = 1.15
        self._resize_job = None
        self.bg_photo = None
        self.chart_photo = None

        # Colors
        self.header_bg = "#0e0f17"
        self.cyan = "#00f2c3"
        self.magenta = "#ff5376"
        self.orange = "#ff8d72"
        self.text_white = "#ffffff"
        self.text_dim = "#c0c5dc"
        self.entry_bg = "#161826"

        # Main Scrollable Canvas
        self.canvas = tk.Canvas(self.root, highlightthickness=0, bg="#000000")
        
        # Sleek vertical scrollbar
        self.scrollbar = tk.Scrollbar(self.root, orient="vertical", command=self.canvas.yview, width=12, bg="#12131f", troughcolor="#000000")
        self.canvas.configure(yscrollcommand=self.scrollbar.set)
        
        self.scrollbar.pack(side="right", fill="y")
        self.canvas.pack(side="left", fill="both", expand=True)

        # Background Image Canvas Item
        self.bg_img_id = self.canvas.create_image(0, 0, anchor="nw")

        # Build UI
        self.build_header()
        self.build_calculator_section()
        self.build_charts_section()

        # Mouse Wheel and Scroll Bindings
        self.root.bind_all("<MouseWheel>", self.on_mousewheel)
        self.root.bind_all("<Button-4>", lambda e: self.canvas.yview_scroll(-2, "units"))
        self.root.bind_all("<Button-5>", lambda e: self.canvas.yview_scroll(2, "units"))
        self.root.bind("<Configure>", self.on_window_configure)

        self.root.update_idletasks()
        self.redraw_layout()
        self.calculate()

    def get_font(self, size, weight="normal"):
        return (self.font_family, max(8, int(size * self.scale)), weight)

    def get_total_content_height(self):
        return int(540 + (410 * self.scale) + 60)

    def on_mousewheel(self, event):
        scroll_units = int(-1 * (event.delta / 120) * 2)
        self.canvas.yview_scroll(scroll_units, "units")

    def scroll_to_charts(self):
        self.canvas.yview_moveto(1.0)

    def scroll_to_top(self):
        self.canvas.yview_moveto(0.0)

    def build_header(self):
        self.header_frame = tk.Frame(self.root, bg=self.header_bg, height=44)
        self.header_win = self.canvas.create_window(14, 10, window=self.header_frame, anchor="nw")

        # Left Title
        self.title_header = tk.Label(
            self.header_frame, text="🎯 Calculadora 3D & Shopee", bg=self.header_bg, fg=self.cyan,
            font=self.get_font(12, "bold")
        )
        self.title_header.pack(side="left", padx=(12, 10), pady=6)

        # Right: TV Zoom Controls
        self.btn_zoom_in = tk.Button(
            self.header_frame, text=" + ", command=lambda: self.adjust_zoom(0.15),
            bg="#222438", fg="#ffffff", font=self.get_font(9, "bold"), relief="flat", cursor="hand2", padx=8, pady=2
        )
        self.btn_zoom_in.pack(side="right", padx=(2, 12), pady=6)

        self.lbl_zoom = tk.Label(
            self.header_frame, text=f"Zoom: {int(self.scale * 100)}%", bg=self.header_bg, fg=self.cyan,
            font=self.get_font(9, "bold"), padx=4
        )
        self.lbl_zoom.pack(side="right", padx=2, pady=6)

        self.btn_zoom_out = tk.Button(
            self.header_frame, text=" - ", command=lambda: self.adjust_zoom(-0.15),
            bg="#222438", fg="#ffffff", font=self.get_font(9, "bold"), relief="flat", cursor="hand2", padx=8, pady=2
        )
        self.btn_zoom_out.pack(side="right", padx=(10, 2), pady=6)

    def build_calculator_section(self):
        # Titles
        self.title_inputs_id = self.canvas.create_text(0, 0, text="⚙️ Parâmetros de Custo & Produção", fill=self.cyan, font=self.get_font(12, "bold"), anchor="w")
        self.title_results_id = self.canvas.create_text(0, 0, text="💵 Resumo Financeiro & Venda", fill=self.magenta, font=self.get_font(12, "bold"), anchor="w")

        # Inputs Setup
        self.fields = [
            ("Tempo de Impressão (horas):", "5", "time"),
            ("Material Usado (gramas):", "120", "material"),
            ("Preço do kg do Filamento (R$):", "125", "price"),
            ("Custo Hora Máquina (R$):", "0.80", "timecost"),
            ("Custo Pintura/Extra (R$):", "0", "paint"),
            ("Multiplicador Lucro (ex: 2.0):", "2.0", "profit"),
            ("Taxa de Falha (ex: 1.1 = +10%):", "1.1", "fail"),
        ]

        self.entry_widgets = {}
        self.entry_label_ids = []
        self.entry_win_ids = []

        for label_text, default_val, key in self.fields:
            lbl_id = self.canvas.create_text(0, 0, text=label_text, fill=self.text_white, font=self.get_font(10), anchor="w")
            self.entry_label_ids.append(lbl_id)

            entry = tk.Entry(
                self.root, width=10, font=self.get_font(10, "bold"),
                bg=self.entry_bg, fg="#ffffff", insertbackground="#ffffff",
                highlightthickness=1, highlightbackground=self.cyan, highlightcolor=self.magenta,
                relief="flat", justify="center"
            )
            entry.insert(0, default_val)
            entry.bind("<Return>", lambda e: self.calculate())

            win_id = self.canvas.create_window(0, 0, window=entry, anchor="e")
            self.entry_widgets[key] = entry
            self.entry_win_ids.append(win_id)

        # Calculate Button
        self.btn_calc = tk.Button(
            self.root, text="⚡ Calcular & Atualizar Tudo", command=self.calculate,
            bg=self.magenta, fg="#ffffff", activebackground="#ff708d", activeforeground="#ffffff",
            font=self.get_font(11, "bold"), relief="flat", cursor="hand2", padx=18, pady=6
        )
        self.btn_calc_win = self.canvas.create_window(0, 0, window=self.btn_calc, anchor="center")

        # Results Items Setup
        self.res_def = [
            ("Custo Total Produção:", "cost", "#ffffff", 11),
            ("Lucro Líquido Real:", "profit_val", self.cyan, 12),
            ("Taxa Retida (Shopee):", "shopee_val", self.orange, 11),
            ("Margem Real:", "margin", self.cyan, 11),
            ("PREÇO FINAL SUGERIDO:", "sale_price", self.cyan, 15),
        ]

        self.res_lbl_ids = []
        self.res_val_ids = {}

        for text, key, color, font_size in self.res_def:
            lbl_id = self.canvas.create_text(0, 0, text=text, fill=self.text_dim, font=self.get_font(10), anchor="w")
            val_id = self.canvas.create_text(0, 0, text="--", fill=color, font=self.get_font(font_size, "bold"), anchor="e")
            self.res_lbl_ids.append(lbl_id)
            self.res_val_ids[key] = val_id

        # Button to scroll down directly to graphics
        self.btn_scroll_charts = tk.Button(
            self.root, text="📉 Rolar para Gráficos ↓", command=self.scroll_to_charts,
            bg="#222438", fg=self.cyan, activebackground="#353855", activeforeground=self.cyan,
            font=self.get_font(10, "bold"), relief="flat", cursor="hand2", padx=14, pady=6
        )
        self.btn_scroll_charts_win = self.canvas.create_window(0, 0, window=self.btn_scroll_charts, anchor="center")

    def build_charts_section(self):
        # Section Title for Charts
        self.title_charts_id = self.canvas.create_text(0, 0, text="📊 Análise Visual & Distribuição de Custos", fill=self.cyan, font=self.get_font(13, "bold"), anchor="w")

        # Back to top button
        self.btn_back_top = tk.Button(
            self.root, text="⬆️ Voltar ao Topo", command=self.scroll_to_top,
            bg="#222438", fg="#ffffff", activebackground="#353855", activeforeground="#ffffff",
            font=self.get_font(9, "bold"), relief="flat", cursor="hand2", padx=10, pady=3
        )
        self.btn_back_top_win = self.canvas.create_window(0, 0, window=self.btn_back_top, anchor="e")

        # In-Memory Headless Matplotlib Figure
        plt.style.use("dark_background")
        self.fig = Figure(dpi=100)
        self.fig.patch.set_facecolor("#10121d")
        self.fig.patch.set_alpha(0.0)
        
        self.canvas_agg = FigureCanvasAgg(self.fig)
        self.ax1 = self.fig.add_subplot(121)
        self.ax2 = self.fig.add_subplot(122)

        # Image item on the Canvas for the chart
        self.chart_canvas_item = self.canvas.create_image(0, 0, anchor="nw")

    def create_css_gradient(self, width, height):
        w = max(10, width)
        h = max(10, height)

        col = Image.new("RGBA", (1, h), (0, 0, 0, 255))
        split_y = int(h * 0.29)
        draw = ImageDraw.Draw(col)

        draw.line([(0, 0), (0, split_y)], fill=(0, 0, 0, 255))

        span = max(1, h - split_y)
        for y in range(split_y, h):
            t = (y - split_y) / float(span)
            r = int(248 * t)
            g = int(94 * t)
            b = int(102 * t)
            draw.point((0, y), fill=(r, g, b, 255))

        return col.resize((w, h), Image.Resampling.BILINEAR)

    def draw_background_image(self, width, height):
        w = max(800, width)
        h = max(self.get_total_content_height(), height)

        base = self.create_css_gradient(w, h)
        overlay = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        draw = ImageDraw.Draw(overlay)

        # 1. Header Glass Box
        safe_rounded_rectangle(draw, [14, 10, w - 14, 54], radius=8, fill=(14, 15, 23, 170), outline=(255, 255, 255, 25), width=1)

        # 2. Top Calculator Cards (y: 64 to 500)
        calc_top = 64
        calc_bottom = 500
        half_w = (w - 38) // 2
        left_x0, left_x1 = 14, 14 + half_w
        right_x0, right_x1 = left_x1 + 10, w - 14

        # Left Card: Inputs (20% opacity translucent glass)
        safe_rounded_rectangle(draw, [left_x0, calc_top, left_x1, calc_bottom], radius=12, fill=(14, 16, 26, 140), outline=(0, 242, 195, 75), width=1)

        # Right Card: Results (20% opacity translucent glass)
        safe_rounded_rectangle(draw, [right_x0, calc_top, right_x1, calc_bottom], radius=12, fill=(14, 16, 26, 140), outline=(255, 83, 118, 75), width=1)

        # Sub-cards inside Results for each row
        start_y = calc_top + 48
        row_h = (calc_bottom - start_y - 60) / 5.0
        for i in range(5):
            sy0 = int(start_y + i * row_h)
            sy1 = int(sy0 + row_h - 6)
            card_fill = (248, 94, 102, 45) if i == 4 else (255, 255, 255, 14)
            card_border = (248, 94, 102, 110) if i == 4 else (255, 255, 255, 20)
            safe_rounded_rectangle(draw, [right_x0 + 12, sy0, right_x1 - 12, sy1], radius=6, fill=card_fill, outline=card_border, width=1)

        # 3. Bottom Charts Card
        chart_top = 516
        chart_bottom = h - 25
        safe_rounded_rectangle(draw, [14, chart_top, w - 14, chart_bottom], radius=12, fill=(14, 16, 26, 150), outline=(0, 242, 195, 70), width=1)

        final_img = Image.alpha_composite(base, overlay)
        self.bg_photo = ImageTk.PhotoImage(final_img)
        self.canvas.itemconfig(self.bg_img_id, image=self.bg_photo)

    def redraw_layout(self):
        width = max(800, self.canvas.winfo_width(), self.root.winfo_width())
        total_h = self.get_total_content_height()

        # Configure Scroll Region
        self.canvas.configure(scrollregion=(0, 0, width, total_h))

        # Render Full-Page Gradient Background
        self.draw_background_image(width, total_h)

        # Header Frame
        self.canvas.coords(self.header_win, 14, 10)
        self.header_frame.configure(width=width - 28)

        # Calculator Cards Positions
        calc_top = 64
        calc_bottom = 500
        half_w = (width - 38) // 2

        left_x0 = 14
        left_x1 = left_x0 + half_w
        right_x0 = left_x1 + 10
        right_x1 = width - 14

        # Left Card (Inputs)
        self.canvas.coords(self.title_inputs_id, left_x0 + 18, calc_top + 24)
        inputs_start_y = calc_top + 50
        row_spacing = (calc_bottom - inputs_start_y - 65) / float(len(self.fields))

        for i, (lbl_id, win_id) in enumerate(zip(self.entry_label_ids, self.entry_win_ids)):
            cy = int(inputs_start_y + i * row_spacing + 10)
            self.canvas.coords(lbl_id, left_x0 + 18, cy)
            self.canvas.coords(win_id, left_x1 - 18, cy)

        btn_y = calc_bottom - 28
        self.canvas.coords(self.btn_calc_win, left_x0 + half_w // 2, btn_y)

        # Right Card (Results)
        self.canvas.coords(self.title_results_id, right_x0 + 18, calc_top + 24)
        start_y = calc_top + 48
        row_h = (calc_bottom - start_y - 60) / 5.0

        for i, (lbl_id, (text, key, color, font_size)) in enumerate(zip(self.res_lbl_ids, self.res_def)):
            sy0 = int(start_y + i * row_h)
            cy = int(sy0 + (row_h - 6) / 2.0)
            val_id = self.res_val_ids[key]

            self.canvas.coords(lbl_id, right_x0 + 24, cy)
            self.canvas.coords(val_id, right_x1 - 24, cy)

        self.canvas.coords(self.btn_scroll_charts_win, right_x0 + (right_x1 - right_x0) // 2, btn_y)

        # Bottom Section (Charts)
        chart_card_top = 516
        self.canvas.coords(self.title_charts_id, 30, chart_card_top + 24)
        self.canvas.coords(self.btn_back_top_win, width - 30, chart_card_top + 24)

        chart_plot_y = chart_card_top + 48
        self.canvas.coords(self.chart_canvas_item, 26, chart_plot_y)

    def adjust_zoom(self, delta):
        new_scale = round(self.scale + delta, 2)
        if 0.8 <= new_scale <= 1.8:
            self.scale = new_scale
            self.lbl_zoom.config(text=f"Zoom: {int(self.scale * 100)}%", font=self.get_font(9, "bold"))
            self.title_header.config(font=self.get_font(12, "bold"))
            self.btn_zoom_in.config(font=self.get_font(9, "bold"))
            self.btn_zoom_out.config(font=self.get_font(9, "bold"))
            self.btn_calc.config(font=self.get_font(11, "bold"))
            self.btn_scroll_charts.config(font=self.get_font(10, "bold"))
            self.btn_back_top.config(font=self.get_font(9, "bold"))

            self.canvas.itemconfig(self.title_inputs_id, font=self.get_font(12, "bold"))
            self.canvas.itemconfig(self.title_results_id, font=self.get_font(12, "bold"))
            self.canvas.itemconfig(self.title_charts_id, font=self.get_font(13, "bold"))

            for lbl_id in self.entry_label_ids:
                self.canvas.itemconfig(lbl_id, font=self.get_font(10))

            for key in self.entry_widgets:
                self.entry_widgets[key].config(font=self.get_font(10, "bold"))

            for lbl_id in self.res_lbl_ids:
                self.canvas.itemconfig(lbl_id, font=self.get_font(10))

            for text, key, color, font_size in self.res_def:
                self.canvas.itemconfig(self.res_val_ids[key], font=self.get_font(font_size, "bold"))

            self.redraw_layout()
            self.calculate()

    def calculate(self):
        try:
            # Inputs
            printing_time = float(self.entry_widgets["time"].get().strip() or 0)
            material_usage = float(self.entry_widgets["material"].get().strip() or 0)
            material_price = float(self.entry_widgets["price"].get().strip() or 0)
            printing_timecost = float(self.entry_widgets["timecost"].get().strip() or 0)
            pintura_cost = float(self.entry_widgets["paint"].get().strip() or 0)
            lucro = float(self.entry_widgets["profit"].get().strip() or 1.0)
            fail_rate = float(self.entry_widgets["fail"].get().strip() or 1.0)

            # Calculations
            nozzle_cost = (material_usage * 25.0) / 5000.0
            material_cost = material_usage * (material_price / 1000.0)
            machine_cost = printing_time * printing_timecost
            custo = machine_cost + nozzle_cost + material_cost + pintura_cost

            valor_base = custo * lucro
            valor_venda_prev = valor_base

            # Shopee Fee Rules
            if valor_venda_prev < 79.99:
                taxa_fixa = 4.0
                taxa_porc = 1.20
            elif 79.99 <= valor_venda_prev < 99.99:
                taxa_fixa = 16.0
                taxa_porc = 1.14
            else:
                taxa_fixa = 20.0
                taxa_porc = 1.14

            valor_venda = round((valor_venda_prev * taxa_porc + taxa_fixa) * fail_rate, 2)
            shopee_cut = round(valor_venda - valor_base, 2)
            lucro_liquido = round(valor_base - custo, 2)
            margem = ((1.0 - (valor_base / valor_venda)) * 100.0) if valor_venda > 0 else 0.0

            # Update Canvas Result Values
            self.canvas.itemconfig(self.res_val_ids["cost"], text=f"R$ {custo:.2f}")
            self.canvas.itemconfig(self.res_val_ids["profit_val"], text=f"R$ {lucro_liquido:.2f}")
            self.canvas.itemconfig(self.res_val_ids["shopee_val"], text=f"R$ {shopee_cut:.2f}")
            self.canvas.itemconfig(self.res_val_ids["margin"], text=f"{margem:.1f}%")
            self.canvas.itemconfig(self.res_val_ids["sale_price"], text=f"R$ {valor_venda:.2f}")

            # Update Charts In-Memory
            self.update_charts(custo, lucro_liquido, shopee_cut, material_cost, machine_cost, nozzle_cost, pintura_cost)

        except (ValueError, KeyError):
            pass

    def update_charts(self, custo, lucro_liq, shopee_tax, mat_cost, mach_cost, noz_cost, paint_cost):
        self.ax1.clear()
        self.ax2.clear()

        self.ax1.set_facecolor("#10121d")
        self.ax2.set_facecolor("#10121d")

        # 1. Donut Chart with External Leader Lines
        labels_pie = ["Custo", "Lucro", "Shopee"]
        values_pie = [max(0.01, custo), max(0.01, lucro_liq), max(0.01, shopee_tax)]
        colors_pie = ["#ff3d60", "#00f2c3", "#ff8d72"]
        total_pie = sum(values_pie)

        wedges, _ = self.ax1.pie(
            values_pie,
            startangle=140,
            colors=colors_pie,
            wedgeprops=dict(width=0.42, edgecolor="#10121d", linewidth=2.5)
        )

        for i, wedge in enumerate(wedges):
            ang = (wedge.theta2 - wedge.theta1) / 2.0 + wedge.theta1
            y = np.sin(np.deg2rad(ang))
            x = np.cos(np.deg2rad(ang))

            ha = "left" if x >= 0 else "right"
            pct = (values_pie[i] / total_pie) * 100.0 if total_pie > 0 else 0.0
            text_content = f"{labels_pie[i]}\n{pct:.1f}%"

            self.ax1.annotate(
                text_content,
                xy=(x * 0.78, y * 0.78),
                xytext=(1.35 * np.sign(x), 1.25 * y),
                horizontalalignment=ha,
                verticalalignment="center",
                fontsize=max(8, int(9.5 * self.scale)),
                fontweight="bold",
                fontfamily=self.font_family,
                color="#ffffff",
                linespacing=1.2,
                arrowprops=dict(
                    arrowstyle="-",
                    connectionstyle=f"angle,angleA=0,angleB={ang}",
                    color=colors_pie[i],
                    lw=1.5,
                    alpha=0.9
                )
            )

        self.ax1.set_title("Composição do Preço Final", fontsize=max(9, int(11 * self.scale)), weight="bold", fontfamily=self.font_family, color="#ffffff", pad=12)

        # 2. Detailed Bar Chart
        categories = ["Filamento", "Máquina", "Bico", "Pintura"]
        cost_values = [mat_cost, mach_cost, noz_cost, paint_cost]
        colors_bar = ["#1d8cf8", "#9b59b6", "#fd5d93", "#00f2c3"]

        bars = self.ax2.bar(categories, cost_values, color=colors_bar, width=0.55)
        self.ax2.set_title("Custos Detalhados (R$)", fontsize=max(9, int(11 * self.scale)), weight="bold", fontfamily=self.font_family, color="#ffffff", pad=12)
        self.ax2.tick_params(axis="x", colors="#ffffff", labelsize=max(8, int(9 * self.scale)))
        self.ax2.tick_params(axis="y", colors=self.text_dim, labelsize=max(7, int(8.5 * self.scale)))
        self.ax2.grid(axis="y", linestyle="--", alpha=0.2, color="#ffffff")

        for spine in self.ax2.spines.values():
            spine.set_visible(False)

        for bar in bars:
            height = bar.get_height()
            if height > 0:
                self.ax2.annotate(
                    f"R${height:.2f}",
                    xy=(bar.get_x() + bar.get_width() / 2, height),
                    xytext=(0, 4),
                    textcoords="offset points",
                    ha="center",
                    va="bottom",
                    fontsize=max(8, int(9 * self.scale)),
                    fontfamily=self.font_family,
                    color="#ffffff",
                    weight="bold"
                )

        # Fill 100% of the graphics card interior proportionally
        width = max(800, self.canvas.winfo_width())
        total_h = self.get_total_content_height()

        chart_card_top = 516
        chart_plot_y = chart_card_top + 48
        chart_bottom = total_h - 25

        fill_w = max(700, width - 52)
        fill_h = max(280, chart_bottom - chart_plot_y - 12)

        self.fig.set_size_inches(fill_w / 100.0, fill_h / 100.0, forward=True)
        self.fig.tight_layout()

        # Render In-Memory
        self.canvas_agg.draw()
        rgba = np.asarray(self.canvas_agg.buffer_rgba())
        chart_img = Image.fromarray(rgba)

        self.chart_photo = ImageTk.PhotoImage(chart_img)
        self.canvas.itemconfig(self.chart_canvas_item, image=self.chart_photo)

    def on_window_configure(self, event=None):
        if self._resize_job is not None:
            self.root.after_cancel(self._resize_job)
        self._resize_job = self.root.after(50, self._perform_resize)

    def _perform_resize(self):
        self._resize_job = None
        self.redraw_layout()
        self.calculate()


if __name__ == "__main__":
    root = tk.Tk()
    app = ScrollableGlassCalculatorApp(root)
    root.mainloop()
