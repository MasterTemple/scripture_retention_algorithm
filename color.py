# Thanks AI
def hex_to_rgb(hex_color):
    """
    Converts a hexadecimal color string to an RGB tuple of integers.

    Args:
        hex_color (str): The hexadecimal color string (e.g., "#RRGGBB" or "RRGGBB").

    Returns:
        tuple: A tuple containing the Red, Green, and Blue integer values (0-255).
    """
    if hex_color.startswith('#'):
        hex_color = hex_color[1:]

    if len(hex_color) != 6:
        raise ValueError("Invalid hex color format. Expected 6 hexadecimal digits.")

    r_hex = hex_color[0:2]
    g_hex = hex_color[2:4]
    b_hex = hex_color[4:6]

    r = int(r_hex, 16)
    g = int(g_hex, 16)
    b = int(b_hex, 16)

    return (r, g, b)

def hex_to_int(hex_color):
    r, g, b = hex_to_rgb(hex_color)
    return (r * 256 * 256) + (g * 256) + b

code = input("Enter hex code: ").strip(" ").lstrip("#")
color = hex_to_int(code)
print(f"Hex '#{code}' converts to Discord Color Integer: '{color}'")
