

from asyncore import loop
from dataclasses import dataclass
from typing import List, Optional, Tuple, Union


@dataclass
class IncrementCell:
    size: int

    def merge(self, other):
        if isinstance(other, IncrementCell):
            self.size += other.size
            return (self, None)
        return (self, other)

    def translate(self) -> List[str]:
        if self.size == 0: return []

        neg = self.size < 0
        size = -self.size if neg else self.size
        if size < 16:
            return [ 
                "    load t2 t0",
                f"    {'isub' if neg else 'iadd'}  t2 {size}",
                "    str  t0 t2",
            ]
        else:
            return [
                f"    imov! t1 {size}",
                "    load t2 t0",
                f"    {'sub' if neg else 'add'}  t2 t1",
                "    str  t0 t2",
            ]

@dataclass
class IncrementPtr:
    size: int

    def merge(self, other):
        if isinstance(other, IncrementPtr):
            self.size += other.size
            return (self, None)
        return (self, other)

    def translate(self) -> List[str]:
        if self.size == 0: return []

        neg = self.size < 0
        size = -self.size if neg else self.size
        if size < 16:
            return [ 
                f"    {'isub' if neg else 'iadd'}  t0 {size}",
            ]
        else:
            return [
                f"    imov! t1 {size}",
                f"    {'sub' if neg else 'add'}  t0 t1",
            ]

@dataclass
class Input:
    def merge(self, other):
        return (self, other)

    def translate(self) -> List[str]:
        return [
            "    call! .key_buffer_read",
            "    str t0 v0",
        ]

@dataclass
class Output:
    def merge(self, other):
        return (self, other)
    
    def translate(self) -> List[str]:
        return [
            "    load p0 t0",
            "    call! .print_char",
        ]

@dataclass
class LoopStart:
    id: int

    def merge(self, other):
        return (self, other)
    
    def translate(self) -> List[str]:
        return [
            "    load t1 t0",
            "    tst  t1",
            f"    jz! .bf_label_end_{self.id}",
            f".bf_label_start_{self.id}",
        ]

@dataclass
class LoopEnd:
    id: int

    def merge(self, other):
        return (self, other)

    def translate(self) -> List[str]:
        return [
            "    load t1 t0",
            "    tst  t1",
            f"    jnz! .bf_label_start_{self.id}",
            f".bf_label_end_{self.id}",
        ]

@dataclass
class Memset:
    def translate(self) -> List[str]:
        return [
            "    str  t0 t3"
        ]

@dataclass
class AddTo:
    offset: int
    do_add: bool
    
    def translate(self) -> List[str]:
        neg = self.offset < 0
        offset = -self.offset if neg else self.offset
        
        if offset < 16:           
            return [
                "    load t1 t0",
                "    mov  t2 t0",
                f"    {'isub' if neg else 'iadd'}  t2 {offset}",
                "    load t4 t2",
                f"    {'add' if self.do_add else 'sub'}  t4 t1",
                "    str  t2 t4"
            ]
        else:
            return [
                "    load t1 t0",
                "    mov  t2 t0",
                f"    imov! t4 {offset}",
                f"    {'sub' if neg else 'add'}  t2 t4",
                "    load t4 t2",
                f"    {'add' if self.do_add else 'sub'}  t4 t1",
                "    str  t2 t4"
            ]


def slurp(file_name: str) -> str:
    with open(file_name) as f:
        return f.read()


def parse_op(c: str) -> Optional[Union[IncrementCell, IncrementPtr, Output, LoopStart, LoopEnd]]:
    if c == "+":
        return IncrementCell(1)
    elif c == "-":
        return IncrementCell(-1)
    elif c == ">":
        return IncrementPtr(1)
    elif c == "<":
        return IncrementPtr(-1)
    elif c == ".":
        return Output()
    elif c == ",":
        return Input()
    elif c == "[":
        return LoopStart(None)
    elif c == "]":
        return LoopEnd(None)
    else:
        return None

def optimize_memset(prog):
    """
    [-] or [+] 
    """
    new_prog = []
    skip = 0
    for i, op in enumerate(prog):
        if skip > 0:
            skip -= 1
            continue

        if not isinstance(op, LoopStart) or i > len(prog) - 3: 
            new_prog.append(op)
            continue

        loop_id = op.id

        if (isinstance(prog[i+1], IncrementCell) 
                and (prog[i+1].size == -1 or prog[i+1].size == 1)
                and isinstance(prog[i+2], LoopEnd)
                and prog[i+2].id == loop_id):
            new_prog.append(Memset())
            skip = 2
        else:
            new_prog.append(op)
    return new_prog

def optimize_add_to(prog):
    """
        [ - > + < ]
        offset = 1
        addTo(offset) => mem[t0 + offset] += mem[t0] 
        memset(t0)
    """

    new_prog = []
    skip = 0
    for i, op in enumerate(prog):
        if skip > 0:
            skip -= 1
            continue

        if not isinstance(op, LoopStart) or i > len(prog) - 6:
            new_prog.append(op)
            continue

        loop_id = op.id

        o1 = prog[i+1]
        o2 = prog[i+2]
        o3 = prog[i+3]
        o4 = prog[i+4]
        o5 = prog[i+5]

        if (isinstance(o1, IncrementCell) and isinstance(o2, IncrementPtr) and 
            isinstance(o3, IncrementCell) and isinstance(o4, IncrementPtr) and
            isinstance(o5, LoopEnd) and
            # check o1 and o3 size are -1 and 1
            o1.size == -1 and (o3.size == -1 or o3.size == 1) and
            # check > and < are equal
            o2.size == -o4.size and
            loop_id == o5.id):

            do_add = o3.size > 0
            offset = o2.size

            new_prog.append(AddTo(offset, do_add))
            new_prog.append(Memset())
            skip = 5
        else:
            new_prog.append(op)
    return new_prog


def main():
    output = """
.reset
    imov! isr .isr
    call! .print_init
    call! .key_buffer_init
    call! .main
    halt

.isr
    isr!
    push! p0 v0

    call! .get_keyboard_ascii
    mov p0 v0
    call! .key_buffer_push
    
    pop! v0 p0
    rti!

#include<../../lib/std/print>
#include<../../lib/std/keyboard>
#include<../../lib/std/arr>

.memory_table   [10240]

.main
    # clear screen
    imov  p0 0
    imov  p1 0
    imov! p2 6000
    call! .arr_memset

    imov! p0 .memory_table
    imov  p1 0
    imov! p2 10240
    call! .arr_memset

    # t0 = mem_ptr
    imov! t0 .memory_table
    imov t1 0
    imov t2 0 # anything
    imov t3 0 # always zero 
    imov t4 0 # anything 
"""

    import argparse
    parser = argparse.ArgumentParser(description="Assembles ToastCPU Architecture")
    parser.add_argument("-i", "--input_file", type=str, required=True, help="Input .tasm file")
    parser.add_argument("-o", "--output_file", type=str, default="a.out", help="Output file location")
    args = parser.parse_args()

    i, o = args.input_file, args.output_file

    print(f"Input: {i}, Output: {o}")

    file_input = slurp(i)
    prog: List[Union[IncrementCell, IncrementPtr, Output, LoopStart, LoopEnd]] = []
    
    prev_op = None

    # collapse consecutive 
    for c in file_input:
        curr_op = parse_op(c)
        if curr_op is None: 
            continue

        if prev_op is None:
            prev_op = curr_op
            prog.append(prev_op)
            continue
        
        (prev_op, curr_op) = prev_op.merge(curr_op)
        if curr_op is not None:
            prog.append(curr_op)
            prev_op = curr_op

    # generate jumps
    starts: List[LoopStart] = []
    loop_id: int = 0
    for op in prog:
        if isinstance(op, LoopStart):
            starts.append(op)
        
        elif isinstance(op, LoopEnd):
            start = starts.pop()
            start.id = loop_id
            op.id = loop_id
            loop_id += 1

    # remove [-] and replace with memset
    prog = optimize_memset(prog)
    prog = optimize_add_to(prog)

    for op in prog:
        for line in op.translate():
            output += "\n" + line
        
    output += "\n" + "    jmpr"
        
    with open(o, "w", encoding="utf-8") as f: 
        f.write(output)
    pass

if __name__=="__main__":
    main()