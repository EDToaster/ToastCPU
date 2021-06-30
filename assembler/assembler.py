from dataclasses import dataclass
from typing import Optional, List, Tuple, Dict
import re

opcodes = {
    "load": "0000",
    "str": "0001",
    "imov": "0010",
    
    "jmp": "1010",
    "jz": "1010",
    "jnz": "1010",
    "jp": "1010",
    "jn": "1010",
    "jmpl": "1010",
    "jzl": "1010",
    "jnzl": "1010",
    "jpl": "1010",
    "jnl": "1010",

    "push": "0101",
    "pop": "0110",
    "halt": "0111",
    "not": "1000",
    "and": "1000",
    "or": "1000",
    "xor": "1000",
    "add": "1000",
    "sub": "1000",
    "mov": "1000",
    "shr": "1000",
    "sshr": "1000",
    "shl": "1000",

    "iand": "1001",
    "ior": "1001",
    "ixor": "1001",
    "iadd": "1001",
    "isub": "1001",
    "ishr": "1001",
    "isshr": "1001",
    "ishl": "1001",
}

one_4bit_opcodes = ["not", ]

two_4bit_opcodes = [
    "load", "str", "and", "or", 
    "xor", "add", "sub", "mov", 
    "shr", "sshr", "shl", "iand", 
    "ior", "ixor", "iadd", "isub", 
    "ishr", "isshr", "ishl"]
imm8_opcodes = [
    "imov"
]

jump_opcodes = {
    "jmp": "00000",
    "jz": "00001",
    "jnz": "00010",
    "jn": "00011",
    "jp": "00100",
    "jmpl": "10000",
    "jzl": "10001",
    "jnzl": "10010",
    "jnl": "10011",
    "jpl": "10100",
}


no_arg_opcodes = [
    "halt"
]

opcodes_suffix = {
    "load": "0000",
    "str": "0000",
    "push": "0000",
    "pop": "0000",
    "not": "0000",
    "and": "0001",
    "or": "0010",
    "xor": "0011",
    "add": "0100",
    "sub": "0101",
    "mov": "0110",
    "shr": "1000",
    "sshr": "1001",
    "shl": "1010",

    "iand": "0001",
    "ior": "0010",
    "ixor": "0011",
    "iadd": "0100",
    "isub": "0101",
    "ishr": "1000",
    "isshr": "1001",
    "ishl": "1010",
}

@dataclass(unsafe_hash=True)
class Label:
    name: str

@dataclass
class Opcode:
    opcode: str

@dataclass
class Register:
    number: int

    def to_binary(self, bits: int) -> str:
        return f"{{:0{bits}b}}".format(self.number)

@dataclass
class Number:
    number: int

    def to_binary(self, bits: int) -> str:
        return f"{{:0{bits}b}}".format(self.number)

@dataclass
class Word:
    text: str
    labels: List[Label]
    word: List
    
    def convert_opcode(self):
        opcode = self.word[0].opcode
        instr = opcodes[opcode]
        if opcode in one_4bit_opcodes:
            # assume word[1] is a 4bit reg or number
            instr += self.word[1].to_binary(4)
            instr += "0" * 4
            instr += opcodes_suffix[opcode]
        elif opcode in two_4bit_opcodes:
            instr += self.word[1].to_binary(4)
            instr += self.word[2].to_binary(4)
            instr += opcodes_suffix[opcode]
        elif opcode in imm8_opcodes:
            instr += self.word[1].to_binary(4)
            instr += self.word[2].to_binary(8)
        elif opcode in jump_opcodes:
            instr += self.word[1].to_binary(4)
            instr += "000"
            instr += jump_opcodes[opcode]
        elif opcode in no_arg_opcodes:
            instr += "0" * 12
        elif opcode == "push":
            instr += "1101"
            instr += self.word[1].to_binary(4)
            instr += "0" * 4
        elif opcode == "pop":
            instr += self.word[1].to_binary(4)
            instr += "11010000"

        return instr

    def to_binary(self) -> str:
        if isinstance(self.word[0], Opcode):
            return self.convert_opcode()
        if isinstance(self.word[0], Number):
            return self.word[0].to_binary(16)

class Program:
    def __init__(self):
        pass

    def parse_label(self, token) -> Optional[Label]:
        x = re.search("^\\.([0-9A-Za-z-_]+)$", token)
        if x is not None:
            return Label(x.group(1))
        else:
            return None
    
    def parse_opcode(self, token) -> Optional[Opcode]:
        if token in opcodes:
            return Opcode(token)

    def parse_register(self, token) -> Optional[Register]:
        x = re.search("^\\[?r([0-9]|1[0-5])\\]?$", token)
        if x is not None:
            return Register(int(x.group(1)))
        else:
            return None

    def parse_number(self, token) -> Optional[Number]:
        try:
            return Number(int(token, 0))
        except ValueError:
            return None

    def parse_token(self, token: str):
        label = self.parse_label(token)
        if label is not None:
            return label

        opcode = self.parse_opcode(token)
        if opcode is not None:
            return opcode

        register = self.parse_register(token)
        if register is not None:
            return register

        number = self.parse_number(token)
        if number is not None:
            return number
        
        return None

    def parse(self, lines: List[str]) -> List[str]:
        raw_format = []
        for line in lines:
            raw = [self.parse_token(tok.strip()) for tok in line.split()]
            if len(raw) < 1 or raw[0] is None:
                continue
            raw_format.append((line.strip(), [r for r in raw if r is not None]))

        # add labels to words
        labeled = []
        current_labels = []
        for (text, line) in raw_format:
            if len(line) < 1:
                continue

            if isinstance(line[0], Label):
                current_labels.append(line[0])
            else:
                labeled.append(Word(text, current_labels, line))
                current_labels = []

        # find label locations
        label_locations: Dict[Label, Number] = {}
        for i, line in enumerate(labeled):
            for label in line.labels:
                label_locations[label] = Number(i)
        
        # replace labels with numbers
        labels_replaced = []
        for line in labeled:
            word = Word(line.text, line.labels, [(label_locations[l] if isinstance(l, Label) else l) for l in line.word])
            labels_replaced.append(word)
        
        [print (line) for line in labels_replaced]

        return [(line.text, line.to_binary()) for line in labels_replaced]

def main():
    import argparse
    parser = argparse.ArgumentParser(description="Assembles ToastCPU Architecture")
    parser.add_argument("-i", "--input_file", type=str, required=True, help="Input .tasm file")
    parser.add_argument("-o", "--output_file", type=str, default="a.out", help="Output file location")
    args = parser.parse_args()

    i, o = args.input_file, args.output_file

    print(f"Input: {i}, Output: {o}")

    lines = None
    with open(i) as f: lines = f.readlines()

    program: Program = Program()
    binary = program.parse(lines)

    with open(o, "w") as f: 
        f.write(
"""DEPTH = 32768;                -- The size of memory in words
WIDTH = 16;                   -- The size of data in bits
ADDRESS_RADIX = HEX;          -- The radix for address values
DATA_RADIX = HEX;             -- The radix for data values
CONTENT                       -- start of (address : data pairs)
BEGIN
""")
        for i, (text, line) in enumerate(binary):
            print(line, text)
            f.write(f"{'{:0>4X}'.format(i)} : {'{:0>4X}'.format(int(line, 2))}; -- {text}\n")

        f.write("END;")


if __name__ == "__main__":
    main()