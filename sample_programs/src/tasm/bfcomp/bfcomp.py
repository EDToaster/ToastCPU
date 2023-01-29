from dataclasses import dataclass
from typing import List, Optional, Tuple, Union, Callable


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
                f"    {'imov!' if size < 256 else 'imov'} t1 {size}",
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
                f"    {'imov!' if size < 256 else 'imov'} t1 {size}",
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
            if neg:       
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
                    f"    load t4 t2 {offset}",
                    f"    {'add' if self.do_add else 'sub'}  t4 t1",
                    f"    str  t2 t4 {offset}"
                ]
        else:
            return [
                "    load t1 t0",
                "    mov  t2 t0",
                f"    {'imov!' if offset < 256 else 'imov'} t4 {offset}",
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

def get_flat_loops(prog) -> List[Tuple[int, int]]:
    start = None
    pairs = []

    for i, op in enumerate(prog):
        if isinstance(op, LoopStart):
            start = i
            continue

        if (isinstance(op, LoopEnd) 
            and start is not None
            and prog[start].id == op.id):
                pairs.append((start, i+1))
                start = None
                continue
    return pairs

def optimize_flat_loops(prog, opt_func: Callable[[List], List]):
    """
    Takes in a list of operations, and an optimizing function
    Returns a program that is optimized for this pass
    """
    new_prog = [op for op in prog]
    
    # assume no overlaps, sort decreasing by loop start
    flat_loops = sorted(get_flat_loops(prog), key=lambda t:t[0], reverse=True)

    for start, end in flat_loops:
        new_prog[start:end] = opt_func(prog[start:end])
    return new_prog


def optimize_memset(prog):
    """
    [-] or [+] 
    """
    def opt_func(slice):
        if len(slice) != 3: return slice

        if slice[1].size != -1 and slice[1].size != 1: return slice

        return [Memset()]

    return optimize_flat_loops(prog, opt_func)

def optimize_add_to(prog):
    """
        [ - > + >> + <<<]
        offset = 1, 3
        addTo(offset) => mem[t0 + offset] += mem[t0] 
        memset(t0)
    """

    def opt_func(slice):
        # slice should have size [6,8,10, ...]
        if len(slice) < 6 or len(slice) % 2 != 0: return slice

        if not isinstance(slice[1], IncrementCell) or slice[1].size != -1: return slice

        # check we have alternating IncrementCell and IncrementPtr
        inner = slice[2:-1]
        check_cell = False

        # keep counts
        offsets: List[int] = []
        offset_sum = 0
        do_adds: List[bool] = []

        for op in inner:
            if check_cell:
                # do not alter the root 
                if offset_sum == 0: 
                    return slice
                # todo: change this in the future to support multadds
                if not isinstance(op, IncrementCell) or (op.size != 1 and op.size != -1): 
                    return slice

                do_adds.append(op.size > 0)
            else:
                if not isinstance(op, IncrementPtr) or op.size == 0: 
                    return slice
                offset_sum += op.size
                offsets.append(offset_sum) 

            check_cell = not check_cell
        
        if offset_sum != 0: return slice

        assert len(offsets) == len(do_adds) + 1

        return [AddTo(offset, do_add) for offset, do_add in zip(offsets, do_adds)] + [Memset()]

    return optimize_flat_loops(prog, opt_func)

def optimize_offset_adds(prog):
    """
        >> -- << 
        p[2] -= 2
    """


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

#include<../../../lib/tasm/print>
#include<../../../lib/tasm/keyboard>
#include<../../../lib/tasm/arr>

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