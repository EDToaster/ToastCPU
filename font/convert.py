import os
from PIL import Image


files = [f for f in os.listdir(".") if f.endswith(".png")]
output = ""
for f in files:
    letter = ord(f[0])
    hexletter = "8'h{:02X}".format(letter)

    # load image
    img = Image.open(f).convert("L")
    WIDTH, HEIGHT = img.size
    assert WIDTH == 8 and HEIGHT == 8
    data = "".join(["1" if p == 0 else "0" for p in list(img.getdata())])
    hexdata = "64'h{:016X}".format(int(data, 2))
    output += f"{hexletter} : bitmap <= {hexdata};\n"

print(output)

ipsum = "Test Text abcdefghijklmnopqrstuvwxyz ABCDEFGHIJKLMNOPQRSTUVWXYZ 0123456789 .,! The Quick Brown Fox jumps over the Lazy Dog!"


for c in ipsum[:2048]:
    print("0x{:04X}".format(ord(c)))