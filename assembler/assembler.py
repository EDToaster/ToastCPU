from __future__ import annotations
from cmath import exp
from dataclasses import dataclass
from typing import Optional, List, Tuple, Dict
import re

opcodes = {
    "load": "0000",
    "str": "0001",
    "imov": "0010",
    "imoh": "0011",
    
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
    
    "jmpr": "1010",
    "jzr": "1010",
    "jnzr": "1010",
    "jpr": "1010",
    "jnr": "1010",

    "rti": "1100",

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
    "ishr", "isshr", "ishl"
]

imm8_opcodes = [
    "imov",
    "imoh",
]

jump_opcodes = {
    "jmp": "000000",
    "jz": "000001",
    "jnz": "000010",
    "jn": "000011",
    "jp": "000100",

    "jmpl": "010000",
    "jzl": "010001",
    "jnzl": "010010",
    "jnl": "010011",
    "jpl": "010100",
}

macros = [ 
    "call", "call!", "push!", "pop!", "load!", "str!",
]
macros += [ f"{j}!" for j in jump_opcodes ]

jumpr_opcodes = {
    "jmpr": "100000",
    "jzr": "100001",
    "jnzr": "100010",
    "jnr": "100011",
    "jpr": "100100",
}


no_arg_opcodes = [
    "halt", "rti"
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

named_registers = {
    "isr": 12,
    "sp": 13,
    "sr": 14,
    "pc": 15,
}

@dataclass(unsafe_hash=True)
class Label:
    name: str

@dataclass
class Opcode:
    opcode: str

@dataclass
class LabelMask:
    """
    Class that represents a certain operation on a label location
    For example:

        call .label

    will be converted to 
        imov r0 LabelMask(.label, 0xFF00, 8)
        ishl r0 4
        ior  r0 LabelMask(.label, 0x00F0, 4)
        ishl r0 4
        ior  r0 LabelMask(.label, 0x000F, 0)
        jmpl r0
    """
    label: Label
    mask: int
    shr: int

    def to_number(self, label_locations: Dict[Label, Number]) -> Number:
        return Number((label_locations[self.label].number & self.mask) >> self.shr)

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
class Instruction:
    text: str
    labels: List[Label]
    words: List
    
    def convert_opcode(self):
        opcode = self.words[0].opcode
        instr = opcodes[opcode]
        if opcode in one_4bit_opcodes:
            # assume word[1] is a 4bit reg or number
            instr += self.words[1].to_binary(4)
            instr += "0" * 4
            instr += opcodes_suffix[opcode]
        elif opcode in two_4bit_opcodes:
            instr += self.words[1].to_binary(4)
            instr += self.words[2].to_binary(4)
            instr += opcodes_suffix[opcode]
        elif opcode in imm8_opcodes:
            instr += self.words[1].to_binary(4)
            instr += self.words[2].to_binary(8)
        elif opcode in jump_opcodes:
            instr += self.words[1].to_binary(4)
            instr += "00"
            instr += jump_opcodes[opcode]
        elif opcode in jumpr_opcodes:
            instr += "000000"
            instr += jumpr_opcodes[opcode]
        elif opcode in no_arg_opcodes:
            instr += "0" * 12
        elif opcode == "push":
            instr += "1101"
            instr += self.words[1].to_binary(4)
            instr += "0" * 4
        elif opcode == "pop":
            instr += self.words[1].to_binary(4)
            instr += "11010000"

        return instr

    def to_binary(self) -> str:
        if isinstance(self.words[0], Opcode):
            return self.convert_opcode()
        if isinstance(self.words[0], Number):
            return self.words[0].to_binary(16)

    def expand_and_convert_mask(self) -> List[Instruction]:
        args = self.words[1:]
        num_args = len(args)
        if isinstance(self.words[0], Opcode):
            opcode = self.words[0].opcode
            if opcode == "call" or opcode == "call!":
                """        
                imov r0 LabelMask(.label, 0x00FF, 0)
                imoh r0 LabelMask(.label, 0xFF00, 8)
                jmpl r0
                """
                label = self.words[1]
                if not isinstance(label, Label):
                    raise "call macro is supposed to have one argument that is a label"
                return [
                    Instruction(self.text, self.labels, [Opcode("imov"), Register(0), LabelMask(label, 0x00FF, 0)]),
                    Instruction("|", [], [Opcode("imoh"), Register(0), LabelMask(label, 0xFF00, 8)]),
                    Instruction("|", [], [Opcode("jmpl"), Register(0)]),
                ]
            elif opcode == "push!":
                return [
                    Instruction(
                        self.text if i == 0 else ("╝" if i == num_args - 1 else "║"), 
                        self.labels if i == 0 else [], 
                        [Opcode("push"), arg]) for i, arg in enumerate(args)
                ]
            elif opcode == "pop!":
                return [
                    Instruction(
                        self.text if i == 0 else ("╝" if i == num_args - 1 else "║"), 
                        self.labels if i == 0 else [], 
                        [Opcode("pop"), arg]) for i, arg in enumerate(args)
                ]
            elif opcode == "load!":
                """
                imov rx LabelMask(.label, 0x00FF, 0)
                imoh rx LabelMask(.label, 0xFF00, 8)
                load rx rx
                """
                reg = self.words[1]
                label = self.words[2]
                assert isinstance(reg, Register) and isinstance(label, Label)
                return [
                    Instruction(self.text, self.labels, [Opcode("imov"), reg, LabelMask(label, 0x00FF, 0)]),
                    Instruction("|", [], [Opcode("imoh"), reg, LabelMask(label, 0xFF00, 8)]),
                    Instruction("|", [], [Opcode("load"), reg, reg])
                ]
            elif opcode == "str!":
                """
                imov r0 LabelMask(.label, 0x00FF, 0)
                imoh r0 LabelMask(.label, 0xFF00, 8)
                str  r0 rx
                """
                label = self.words[1]
                reg = self.words[2]
                assert isinstance(reg, Register) and isinstance(label, Label)
                return [
                    Instruction(self.text, self.labels, [Opcode("imov"), Register(0), LabelMask(label, 0x00FF, 0)]),
                    Instruction("|", [], [Opcode("imoh"), Register(0), LabelMask(label, 0xFF00, 8)]),
                    Instruction("|", [], [Opcode("str"), Register(0), reg])
                ]
            elif opcode[:-1] in jump_opcodes and opcode[-1] == '!':
                """
                imov r0 LabelMask(.label, 0x00FF, 0)
                imoh r0 LabelMask(.label, 0xFF00, 8)
                j_op r0
                """
                label = self.words[1]
                assert isinstance(label, Label)
                return [
                    Instruction(self.text, self.labels, [Opcode("imov"), Register(0), LabelMask(label, 0x00FF, 0)]),
                    Instruction("|", [], [Opcode("imoh"), Register(0), LabelMask(label, 0xFF00, 8)]),
                    Instruction("|", [], [Opcode(opcode[:-1]), Register(0)])
                ]
            else:
                return [
                    Instruction(self.text, self.labels, [(LabelMask(l, 0xFFFF, 0) if isinstance(l, Label) else l) for l in self.words])
                ]
        if isinstance(self.words[0], Number):
            return [
                self
            ]

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

    def parse_macro(self, token) -> Optional[Opcode]:
        if token in macros:
            return Opcode(token)

    def parse_register(self, token) -> Optional[Register]:
        x = re.search("^\\[?r([0-9]|1[0-5])\\]?$", token)
        if x is not None:
            num = int(x.group(1))
            if num == 0:
                raise "r0 is reserved for the assembler"
            return Register(num)
        elif token in named_registers:
            return Register(named_registers[token])
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

        macro = self.parse_macro(token)
        if macro is not None:
            return macro

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
            stripped_line = line.strip()
            if stripped_line.startswith("\"") and stripped_line.endswith("\""):
                raw_string = stripped_line[1:-1]
                raw_format += [(c, [Number(ord(c))]) for c in raw_string]
                continue

            raw = [self.parse_token(tok.strip()) for tok in line.split()]
            if len(raw) < 1 or raw[0] is None:
                continue
            raw_format.append((stripped_line, [r for r in raw if r is not None]))

        # add labels to words
        label_locations = {}
        labeled_instructions: List[Instruction] = []
        current_labels_for_line = []
        for (text, line) in raw_format:
            if len(line) < 1:
                continue

            if len(line) >= 2 and isinstance(line[0], Label) and isinstance(line[1], Number):
                label_locations[line[0]] = line[1]
            elif isinstance(line[0], Label):
                current_labels_for_line.append(line[0])
            else:
                labeled_instructions.append(Instruction(text, current_labels_for_line, line))
                current_labels_for_line = []

        # do macro expansion -- also convert label (in tokens, not current labels for line) to label masks
        expanded_instructions = []
        for instruction in labeled_instructions:
            expanded_instructions += instruction.expand_and_convert_mask()
            
        
        print()
        print()
        print()
        [print (line) for line in expanded_instructions]

        # find label locations
        for i, line in enumerate(expanded_instructions):
            for label in line.labels:
                label_locations[label] = Number(i)
        
        for label, loc in label_locations.items():
            print (f"{label} at {loc}")

        # replace labels with numbers
        labels_replaced = []
        for line in expanded_instructions:
            word = Instruction(line.text, line.labels, [(l.to_number(label_locations) if isinstance(l, LabelMask) else l) for l in line.words])
            labels_replaced.append(word)
        
        print()
        print()
        print()
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