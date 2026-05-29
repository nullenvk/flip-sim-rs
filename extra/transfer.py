from PIL import Image

data = Image.open('/ram/data.jpg')

output = []
for y in range(0, 96):
    for x in range(0, 96, 2):
        l = 0
        for i in range(2):
            px = data.getpixel((x + i, y))
            v = 0b1111 if px[0] > 0 else 0b0000
            l |= v << ((1 - i) * 4)
        output.append(l)

from q import wb
wb('/ram/raw', bytes(output))
