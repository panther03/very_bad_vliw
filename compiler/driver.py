import sys
import subprocess
import tempfile
import re

CC_PREFIX="riscv64-elf-"

abi = {
    "zero" : "x0",
    "ra" : "x1",
    "sp" : "x2",
    "gp" : "x3",
    "tp" : "x4",
    "t0" : "x5",
    "t1" : "x6",
    "t2" : "x7",
    "fp" : "x8",
    "s0" : "x8",
    "s1" : "x9",
    "a0" : "x10",
    "a1" : "x11",
    "a2" : "x12",
    "a3" : "x13",
    "a4" : "x14",
    "a5" : "x15",
    "a6" : "x16",
    "a7" : "x17",
    "s2" : "x18",
    "s3" : "x19",
    "s4" : "x20",
    "s5" : "x21",
    "s6" : "x22",
    "s7" : "x23",
    "s8" : "x24",
    "s9" : "x25",
    "s10": "x26",
    "s11": "x27",
    "t3" : "x28",
    "t4" : "x29",
    "t5" : "x30",
    "t6" : "x31"
}

def parse_objdump(objdump_output):
    asm_lines = []
    for objdump_line in objdump_output.splitlines():
        # separator used by objdump
        asm_split = objdump_line.split("          \t")
        if len(asm_split) == 2:
            asm_line_raw = asm_split[1]
            if (i := asm_line_raw.find('#')) >= 0:
                asm_line_raw = asm_line_raw[:i]
            if (i := asm_line_raw.find('<')) >= 0:
                asm_line_raw = asm_line_raw[:i]
            asm_line_raw_split = asm_line_raw.split('\t')
            op = asm_line_raw_split[0]
            operands = asm_line_raw_split[1] if len(asm_line_raw_split) == 2 else ""
            operands_after = ""
            if op.startswith('b') or op.startswith('j'):
                operands = operands.split(',')
                operands_after = operands[-1]
                operands = operands[:-1]
                operands = ",".join(operands)
            for (abi_name, x_name) in abi.items().__reversed__():
                operands = operands.replace(abi_name, x_name)
            if op.startswith('b'):
                operands += ","
            if operands_after:
                operands += operands_after
            if op.startswith('b'):
                # branch instruction; add 0x before the last operand
                operands = operands.split(',')
                if op.endswith('z'):
                    operands.insert(1, "x0")
                    op = op[:-1]
                operands[2] = "0x" + operands[2]
                operands = ",".join(operands)
            elif op.startswith('j'):
                operands = "0x" + operands
            asm_lines.append(op + " " + operands)
    return '\n'.join(asm_lines)

if __name__ == "__main__":
    assert len(sys.argv) == 3
    input_elf = sys.argv[1]
    output_hex = sys.argv[2]

    objdump_output = subprocess.run([CC_PREFIX + "objdump", "-d", input_elf], capture_output=True)
    input_asm = parse_objdump(objdump_output.stdout.decode('utf-8'))
    print(input_asm)
    temp_out_asm = tempfile.mktemp()
    subprocess.run(["target/release/hw2", "STDIN", "out.asm"], input=input_asm.encode("utf-8"))
