import os
from PIL import Image, ImageFont, ImageDraw

image = Image.open("./Curses_640x300diag.png")
width, height = image.size
print(image.size)
# take 8x12 chunks

output = ""
char = 0
for j in range(height//12):
    for i in range(width//8):
        hexletter = "8'h{:02X}".format(char)
        chunk = image.crop((i*8, j*12, i*8 + 8, j*12 + 12)).convert('1')
        chunk.show()
        data = "".join(["0" if p == 0 else "1" for p in list(chunk.getdata())])
        hexdata = "96'h{:024X}".format(int(data, 2))
        output += f"{hexletter} : bitmap = {hexdata};\n"
        char += 1
# files = [f for f in os.listdir(".") if f.endswith(".png")]
# for c in chars:
#     letter = ord(c)
#     hexletter = "8'h{:02X}".format(letter)

#     print(font.getsize(c))
#     # load image
#     image = Image.new("RGBA", (24,36), (255,255,255))
#     draw = ImageDraw.Draw(image)

#     draw.text((0, 6), c, (0, 0, 0), font=font)
#     image = image.resize((8, 12),resample=Image.NEAREST).convert('1')
#     image.show()

#     WIDTH, HEIGHT = image.size
#     assert WIDTH == 8 and HEIGHT == 12
#     data = "".join(["1" if p == 0 else "0" for p in list(image.getdata())])
#     hexdata = "96'h{:024X}".format(int(data, 2))
#     output += f"{hexletter} : bitmap = {hexdata};\n"

print(output)

ipsum = "Test Text abcdefghijklmnopqrstuvwxyz ABCDEFGHIJKLMNOPQRSTUVWXYZ 0123456789 .,! The Quick Brown Fox jumps over the Lazy Dog! Cat ipsum dolor sit amet, see brother cat receive pets, attack out of jealousy purr purr purr until owner pets why owner not pet me hiss scratch meow. Sleep. Plan steps for world domination catch mouse and gave it as a present but the fat cat sat on the mat bat away with paws purrrrrr show belly. Stinky cat run in circles, and eat fish on floor hiss and stare at nothing then run suddenly away but furball roll roll roll for headbutt owner's knee. Love fish push your water glass on the floor rub face on everything, and spill litter box, scratch at owner, destroy all furniture, especially couch, nap all day i hate cucumber pls dont throw it at me so ignore the squirrels, you'll never catch them anyway. Litter box is life the cat was chasing the mouse. Destroy couch as revenge pooping rainbow while flying in a toasted bread costume in space but sun bathe cats are fats i like to pets them they like to meow back but i see a bird i stare at it i meow at it i do a wiggle come here birdy. Crusty butthole i show my fluffy belly but it's a trap! if you pet it i will tear up your hand. I shredded your linens for you. Human give me attention meow i like fish lick the other cats. Stare at owner accusingly then wink fight own tail, step on your keyboard while you're gaming and then turn in a circle . Catty ipsum sleeping in the box hiss and stare at nothing then run suddenly away. Jump on human and sleep on her all night long be long in the bed, purr in the morning and then give a bite to every human around for not waking up request food, purr loud scratch the walls, the floor, the windows, the humans try to jump onto window and fall while scratching at wall or you are a captive audience while sitting on the toilet, pet me good now the other hand, too spot something, big eyes, big eyes, crouch, shake butt, prepare to pounce scream for no reason at 4 am. Cat ass trophy walk on keyboard or go into a room to decide you didn't want to be in there anyway so lick left leg for ninety minutes, still dirty. Twitch tail in permanent irritation when in doubt, wash. Cats woo chase laser. Purr when being pet no, you can't close the door, i haven't decided whether or not i wanna go out so hiss and stare at nothing then run suddenly away scratch at fleas, meow until belly rubs, hide behind curtain when vacuum cleaner is on scratch strangers and poo on owners food. Human give me attention meow fall over dead (not really but gets sypathy) but cat walks in keyboard and pooping rainbow while flying in a toasted bread costume in space. Ask for petting. Meowwww you are a captive audience while sitting on the toilet, pet me, chirp at birds crash against wall but walk away like nothing happened find empty spot in cupboard and sleep all day poop on couch. Get suspicious of own shadow then go play with toilette paper do doodoo in the litter-box, clickityclack on the piano, be frumpygrumpy make plans to dominate world and then take a nap. Attack dog, run away and pretend to be victim somehow manage to catch a bird but have no idea what to do next, so play with it until it dies of shock yet drink water out of the faucet but sun bathe mew mew."

import random as r

for c in ipsum[:64*40]:
    print("0b00{:03b}{:03b}{:08b}".format(0, 7, ord(c)))