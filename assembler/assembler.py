from __future__ import annotations
from cmath import exp
from dataclasses import dataclass
from typing import Optional, List, Set, Tuple, Dict
import re
import os

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
    "cmp": "1000",
    "shr": "1000",
    "sshr": "1000",
    "shl": "1000",

    "iand": "1001",
    "ior": "1001",
    "ixor": "1001",
    "iadd": "1001",
    "isub": "1001",
    "icmp": "1001",
    "tst": "1001",
    "ishr": "1001",
    "isshr": "1001",
    "ishl": "1001",
}

one_4bit_opcodes = ["not", "tst"]

two_4bit_opcodes = [
    "and", "or", 
    "xor", "add", "sub", "mov", "cmp",
    "shr", "sshr", "shl", "iand", 
    "ior", "ixor", "iadd", "isub", "icmp",
    "ishr", "isshr", "ishl"
]

load_str_opcodes = [
    "load", "str"
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
    "call", "call!", "push!", "pop!", 
    "load!", "str!", "imov!",
    "isr!", "rti!",

    "icmp!",
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
    "push": "0000",
    "pop": "0000",
    "not": "0000",
    "and": "0001",
    "or": "0010",
    "xor": "0011",
    "add": "0100",
    "sub": "0101",
    "mov": "0110",
    "cmp": "0111",
    "shr": "1000",
    "sshr": "1001",
    "shl": "1010",

    "iand": "0001",
    "ior": "0010",
    "ixor": "0011",
    "iadd": "0100",
    "isub": "0101",
    "icmp": "0111",
    "tst": "0111",
    "ishr": "1000",
    "isshr": "1001",
    "ishl": "1010",
}

named_registers = {
    "ar": 0,

    "p0": 1,
    "p1": 2,
    "p2": 3,
    "p3": 4,

    "v0": 5,

    "t0": 6,
    "t1": 7,
    "t2": 8,
    "t3": 9,
    "t4": 10,
    "t5": 11,

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
class Allocation:
    size: int

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
        elif opcode in load_str_opcodes:
            instr += self.words[1].to_binary(4)
            instr += self.words[2].to_binary(4)
            if len(self.words) > 3:
                instr += self.words[3].to_binary(4)
            else:
                instr += "0" * 4
        elif opcode in two_4bit_opcodes:
            instr += self.words[1].to_binary(4)
            instr += self.words[2].to_binary(4)
            instr += opcodes_suffix[opcode]
        elif opcode in imm8_opcodes:
            instr += self.words[1].to_binary(4)
            instr += self.words[2].to_binary(8)
        elif opcode in jump_opcodes:
            instr += self.words[1].to_binary(4)
            instr += "0" * 2
            instr += jump_opcodes[opcode]
        elif opcode in jumpr_opcodes:
            instr += "0" * 6
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
                    Instruction(f". imov r0 .{label.name}  [{self.text}]", self.labels, [Opcode("imov"), Register(0), LabelMask(label, 0x00FF, 0)]),
                    Instruction(f"| imoh r0 .{label.name}", [], [Opcode("imoh"), Register(0), LabelMask(label, 0xFF00, 8)]),
                    Instruction(f"' jmpl", [], [Opcode("jmpl"), Register(0)]),
                ]
            elif opcode == "push!":
                return [
                    Instruction(
                        f". push [{self.text}]" if i == 0 else ("' push" if i == num_args - 1 else "| push"), 
                        self.labels if i == 0 else [], 
                        [Opcode("push"), arg]) 
                    for i, arg in enumerate(args)
                ]
            elif opcode == "pop!":
                return [
                    Instruction(
                        f". pop [{self.text}]" if i == 0 else ("' pop" if i == num_args - 1 else "| pop"), 
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
                    Instruction(f". imov [{self.text}]", self.labels, [Opcode("imov"), reg, LabelMask(label, 0x00FF, 0)]),
                    Instruction("| imoh", [], [Opcode("imoh"), reg, LabelMask(label, 0xFF00, 8)]),
                    Instruction("' load", [], [Opcode("load"), reg, reg])
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
                    Instruction(f". imov [{self.text}]", self.labels, [Opcode("imov"), Register(0), LabelMask(label, 0x00FF, 0)]),
                    Instruction("| imoh", [], [Opcode("imoh"), Register(0), LabelMask(label, 0xFF00, 8)]),
                    Instruction("' str", [], [Opcode("str"), Register(0), reg])
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
                    Instruction(f". imov [{self.text}]", self.labels, [Opcode("imov"), Register(0), LabelMask(label, 0x00FF, 0)]),
                    Instruction("| imoh", [], [Opcode("imoh"), Register(0), LabelMask(label, 0xFF00, 8)]),
                    Instruction("' j_op", [], [Opcode(opcode[:-1]), Register(0)])
                ]
            elif opcode == "imov!":
                """
                imov rx LabelMask(.label, 0x00FF, 0)
                imoh rx LabelMask(.label, 0xFF00, 8)
                """
                reg = self.words[1]
                dest = self.words[2]
                assert isinstance(reg, Register) and (isinstance(dest, Label) or isinstance(dest, Number))
                is_label = isinstance(dest, Label)
                return [
                    Instruction(f". imov [{self.text}]", self.labels, [Opcode("imov"), reg, LabelMask(dest, 0x00FF, 0) if is_label else Number(dest.number & 0x00FF)]),
                    Instruction("' imoh", [], [Opcode("imoh"), reg, LabelMask(dest, 0xFF00, 8) if is_label else Number((dest.number & 0xFF00) >> 8)]),
                ]
            elif opcode == "isr!":
                """
                push r0
                """
                return [ 
                    Instruction(f"· push r0 [{self.text}]", self.labels, [Opcode("push"), Register(0)])
                ]
            elif opcode == "rti!":
                """
                pop r0
                rti
                """
                return [
                    Instruction(f". pop r0 [{self.text}]", self.labels, [Opcode("pop"), Register(0)]),
                    Instruction("' rti", [], [Opcode("rti")]),
                ]
            elif opcode == "icmp!":
                """
                imov r0 LabelMask(.label, 0x00FF, 0)
                imoh r0 LabelMask(.label, 0xFF00, 8)
                cmp  rx r0
                """
                reg = self.words[1]
                dest = self.words[2]
                assert isinstance(reg, Register) and (isinstance(dest, Label) or isinstance(dest, Number))
                is_label = isinstance(dest, Label)
                return [
                    Instruction(f". imov [{self.text}]", self.labels, [Opcode("imov"), Register(0), LabelMask(dest, 0x00FF, 0) if is_label else Number(dest.number & 0x00FF)]),
                    Instruction("| imoh", [], [Opcode("imoh"), Register(0), LabelMask(dest, 0xFF00, 8) if is_label else Number((dest.number & 0xFF00) >> 8)]),
                    Instruction("' cmp", [], [Opcode("cmp"), reg, Register(0)]),
                ]
            else:
                return [
                    Instruction("  " + self.text, self.labels, [(LabelMask(l, 0xFFFF, 0) if isinstance(l, Label) else l) for l in self.words])
                ]
        if isinstance(self.words[0], Number):
            return [
                self
            ]

@dataclass
class Ignore:
    pass

class Program:
    def __init__(self):
        pass

    def parse_ignore(self, token) -> Optional[Ignore]:
        if token == "fn": return Ignore()
        return None

    def parse_label(self, token) -> Optional[Label]:
        x = re.search("^\\.([0-9A-Za-z-_]+)$", token)
        if x is not None:
            return Label(x.group(1))
        else:
            return None
    
    def parse_opcode(self, token) -> Optional[Opcode]:
        # todo: should make ISR more automated using labels
        if token == "rti":
            raise "The rti instruction isn't meant to be used when using the assembler. Use the isr! and rti! macros instead to automatically push and pop the AR register."

        if token in opcodes:
            return Opcode(token)

    def parse_macro(self, token) -> Optional[Opcode]:
        if token in macros:
            return Opcode(token)

    def parse_register(self, token) -> Optional[Register]:
        x = re.search("^\\[?r([0-9]|1[0-5])\\]?$", token)
        num = None
        if x is not None:
            num = int(x.group(1))
        elif token in named_registers:
            num = named_registers[token]
        else:
            return None
            
        if num == 0:
            raise "r0 is reserved for the assembler"
        return Register(num)

    def parse_number(self, token) -> Optional[Number]:
        if len(token) == 3 and token[0] == '\'' and token[-1] == '\'':
            return Number(ord(token[1]))

        try:
            return Number(int(token, 0))
        except ValueError:
            return None

    def parse_allocation(self, token) -> Optional[Allocation]:
        try:
            if token[0] == '[' and token[-1] == ']':
                return Allocation(int(token[1:-1], 0))
        except ValueError:
            return None
        return None

    def parse_token(self, token: str):
        ignore = self.parse_ignore(token)
        if ignore is not None:
            return ignore

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

        alloc = self.parse_allocation(token)
        if alloc is not None:
            return alloc
        
        return None

    def parse(self, lines: List[str]) -> List[str]:
        raw_format = []
        for line in lines:
            stripped_line = line.strip()
            line_to_parse = re.split("#|@|//", stripped_line)[0].strip()

            # parse strings!
            if line_to_parse.startswith("\"") and line_to_parse.endswith("\""):
                raw_string = line_to_parse[1:-1]
                raw_format += [(str(c.encode('ascii'))[2:-1], [Number(ord(c))]) for c in raw_string.encode('utf-8').decode('unicode_escape')]
                continue

            raw = [self.parse_token(tok.strip()) for tok in line_to_parse.split()]
            if len(raw) > 0 and isinstance(raw[0], Ignore):
                raw.pop(0) 

            if len(raw) < 1 or raw[0] is None:
                continue

            raw_format.append((stripped_line, [r for r in raw if r is not None]))

        # add labels to words
        label_locations = {}
        labeled_instructions: List[Instruction] = []
        current_labels_for_line = []

        # compile-time allocation
        heap = 0x8000

        for (text, line) in raw_format:
            if len(line) < 1:
                continue

            if len(line) >= 2 and isinstance(line[0], Label) and isinstance(line[1], Number):
                label_locations[line[0]] = line[1]
            elif len(line) >= 2 and isinstance(line[0], Label) and isinstance(line[1], Allocation):
                label_locations[line[0]] = Number(heap)
                heap += line[1].size
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

def read_raw_lines(file_name: str) -> List[str]:
    with open(file_name) as f:
        return f.readlines()

def preprocess(root_file: str, included: Set[str]) -> List[str]:
    """
    Read in the file and preprocess annotations and include statements
    """
    include_regex = re.compile(r"\s*#include<([\w\-. /]+)>")

    lines: List[str] = read_raw_lines(root_file)
    print(f"Preprocessing file {root_file}")

    final_lines = []
    for l in lines:
        match = re.match(include_regex, l)
        if match is not None:
            to_include = os.path.abspath(f"{match.group(1)}.tasm")
            if to_include in included:
                continue
        
            included.add(to_include)
            final_lines += preprocess(to_include, included)
        else:
            final_lines.append(l)
    
    return final_lines


def main():
    import argparse
    parser = argparse.ArgumentParser(description="Assembles ToastCPU Architecture")
    parser.add_argument("-i", "--input_file", type=str, required=True, help="Input .tasm file")
    parser.add_argument("-o", "--output_file", type=str, default="a.out", help="Output file location")
    args = parser.parse_args()

    i, o = args.input_file, args.output_file

    print(f"Input: {i}, Output: {o}")

    input_file = os.path.abspath(i)
    lines = preprocess(input_file, set([input_file]))

    for l in lines:
        print(l.strip())

    program: Program = Program()
    binary = program.parse(lines)

    with open(o, "w", encoding="utf-8") as f: 
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