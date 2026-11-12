// The machine code of a firmware binary, translated to C one instruction at a time, from the
// meaning Ghidra's processor description gives each instruction (its p-code). Nothing is
// guessed and nothing is tidied: a register is a cell of an array, a jump is a goto, a call is
// a call. What the translation does not cover (a computed jump, an operation it does not know)
// sets `arclang_uncovered` where it would run, so that a proof over this C never passes on it.
//
//   analyzeHeadless <project dir> <name> -import <image> -processor <language> -cspec gcc \
//       -loader BinaryLoader -scriptPath <dir of this file> \
//       -postScript Lift.java <out.c> [<address of a function>...]
//
// Addresses are byte addresses in the flash, as a disassembler prints them. The C needs
// "arclang_lifted.h" (written by `arclang prove`): the memory, the registers and the flash.
//
//@category ArcLang
import java.io.PrintWriter;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeSet;

import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.address.AddressSetView;
import ghidra.program.model.address.AddressSpace;
import ghidra.program.model.lang.Register;
import ghidra.program.model.listing.Function;
import ghidra.program.model.listing.Instruction;
import ghidra.program.model.mem.MemoryBlock;
import ghidra.program.model.pcode.PcodeOp;
import ghidra.program.model.pcode.Varnode;

public class Lift extends GhidraScript {
    private AddressSpace code;
    /** The functions the function being translated calls or jumps to. */
    private Set<String> calls = new TreeSet<>();
    /** What is said of the function being translated, for who looks for its functions by their shape. */
    private StringBuilder sites = new StringBuilder();
    /** The registers r0..r31 that hold a constant where the translation is, and the instruction it is at. */
    private final Map<Long, Long> constants = new HashMap<>();
    private String place = "";
    /**
     * The head of the cycle the code never leaves, when the proof is of that cycle and not of a
     * function. The translation stops there (`arclang_cut()`), and can be entered there again
     * (`arclang_resume`): from the reset to the first arrival is the start, from one arrival to
     * the next is one turn.
     */
    private Address cut = null;
    private final Set<String> userOps = new TreeSet<>();

    @Override
    public void run() throws Exception {
        String[] args = getScriptArgs();
        if (args.length < 1) {
            throw new IllegalArgumentException("usage: Lift.java <output file> [<address of a function>...]");
        }
        code = currentProgram.getAddressFactory().getDefaultAddressSpace();
        Address first = currentProgram.getMinAddress();
        // A raw image names no entry point: a firmware image starts at its first byte.
        entry(first);
        for (int k = 1; k < args.length; k++) {
            // `cut:0x0390`: the place a cycle is cut at (see `cut`); any other argument is where a function is.
            if (args[k].startsWith("cut:")) {
                cut = code.getAddress(Long.decode(args[k].substring(4)), false);
            } else {
                entry(code.getAddress(Long.decode(args[k]), false));
            }
        }
        analyzeAll(currentProgram);
        if (cut != null && (currentProgram.getFunctionManager().getFunctionContaining(cut) == null || currentProgram.getListing().getInstructionAt(cut) == null)) {
            throw new IllegalArgumentException("no instruction of a function at the cut 0x" + hex(cut));
        }

        List<Function> functions = new ArrayList<>();
        for (Function function : currentProgram.getFunctionManager().getFunctions(true)) {
            functions.add(function);
        }
        StringBuilder bodies = new StringBuilder();
        StringBuilder facts = new StringBuilder();
        for (Function function : functions) {
            translate(function, bodies, facts);
        }
        try (PrintWriter out = new PrintWriter(args[0], "UTF-8")) {
            out.println("/* Machine code translated to C by arclang (Lift.java), one instruction at a time. */");
            out.println("//@ language " + currentProgram.getLanguageID());
            out.println("//@ start 0x" + Long.toHexString(first.getOffset()));
            out.println("//@ memory 0x" + Long.toHexString(RAM_FIRST) + " 0x" + Long.toHexString(RAM_LAST));
            for (Register register : currentProgram.getLanguage().getRegisters()) {
                if (register.isBaseRegister() || register.getMinimumByteSize() <= 2) {
                    out.println("//@ register " + register.getName() + " " + register.getAddressSpace().getName()
                        + " 0x" + Long.toHexString(register.getOffset()) + " " + register.getMinimumByteSize());
                }
            }
            for (String name : userOps) {
                out.println("//@ operation " + name);
            }
            out.print(facts);
            out.println("#include <stdint.h>");
            flash(out);
            out.println("#include \"arclang_lifted.h\"");
            for (Function function : functions) {
                String at = hex(function.getEntryPoint());
                out.println("void f_" + at + "(void);");
                out.println("#ifndef c_" + at);
                out.println("#define c_" + at + " f_" + at);
                out.println("#endif");
            }
            for (String name : userOps) {
                out.println("void arclang_op_" + name + "(void);");
            }
            out.print(bodies);
        }
    }

    private void entry(Address at) throws Exception {
        if (currentProgram.getFunctionManager().getFunctionAt(at) == null) {
            disassemble(at);
            createFunction(at, null);
        }
    }

    private static String hex(Address address) {
        return String.format("%04x", address.getOffset());
    }

    /** The flash as the image gives it, read by the instructions that load from it. */
    private void flash(PrintWriter out) throws Exception {
        long low = Long.MAX_VALUE;
        long high = -1;
        for (MemoryBlock block : currentProgram.getMemory().getBlocks()) {
            if (block.getStart().getAddressSpace().equals(code) && block.isInitialized()) {
                low = Math.min(low, block.getStart().getOffset());
                high = Math.max(high, block.getEnd().getOffset() + code.getAddressableUnitSize() - 1);
            }
        }
        out.println("#define ARCLANG_FLASH_BASE 0x" + Long.toHexString(low) + "u");
        out.println("#define ARCLANG_FLASH_SIZE " + (high - low + 1) + "u");
        out.print("static const uint8_t arclang_flash[ARCLANG_FLASH_SIZE] = {");
        for (long at = low; at <= high; at++) {
            if ((at - low) % 16 == 0) {
                out.print("\n   ");
            }
            // One byte of the flash: the memory is read by byte, whatever the unit the space counts in.
            int value;
            try {
                value = currentProgram.getMemory().getByte(code.getAddress(at, false)) & 0xff;
            } catch (Exception absent) {
                value = 0xff;
            }
            out.print(String.format(" 0x%02x,", value));
        }
        out.println("\n};");
    }

    private static String mask(int size) {
        return size >= 8 ? "UINT64_C(0xffffffffffffffff)" : String.format("UINT64_C(0x%x)", (1L << (8 * size)) - 1);
    }

    /** A value read, as an unsigned 64-bit expression that holds no bit above its size. */
    private String read(Varnode v) {
        if (v.isConstant()) {
            long value = v.getSize() >= 8 ? v.getOffset() : v.getOffset() & ((1L << (8 * v.getSize())) - 1);
            return String.format("UINT64_C(0x%x)", value);
        }
        if (v.isUnique()) {
            return String.format("(u_%x & %s)", v.getOffset(), mask(v.getSize()));
        }
        if (v.isRegister() || v.getAddress().getAddressSpace().getName().equals("mem")) {
            // A place known where the code is written is read where it is, byte by byte, low byte first:
            // a prover follows a cell of an array far better than a call that finds it.
            StringBuilder value = new StringBuilder("(");
            for (int k = 0; k < v.getSize(); k++) {
                value.append(k == 0 ? "" : " | ").append("(uint64_t)").append(cell(v, k, true)).append(k == 0 ? "" : " << " + (8 * k));
            }
            return value.append(")").toString();
        }
        return String.format("arclang_flash_rd(0x%xu, %d)", v.getOffset(), v.getSize());
    }

    /** The memory of the chip that is plain memory: its RAM (an ATmega328P's). Below are the registers r0..r31 and the registers of the chip. */
    private static final long RAM_FIRST = 0x100;
    private static final long RAM_LAST = 0x8ff;

    /**
     * The byte `k` of a place of the registers or of the memory: the cell of the array that holds
     * it, or the call that reads it when it is a register of the chip or lies outside the memory.
     * `null` for a write that has to go through a call.
     */
    private String cell(Varnode v, int k, boolean read) {
        long at = v.getOffset() + k;
        if (v.isRegister()) {
            return String.format("arclang_reg[0x%xu]", at & 0xff);
        }
        if (at < 32) {
            return String.format("arclang_r[%d]", at);
        }
        if (at >= RAM_FIRST && at <= RAM_LAST) {
            return String.format("arclang_mem[0x%xu]", at);
        }
        return read ? String.format("arclang_mem_rd1(0x%xu)", at) : null;
    }

    private String write(Varnode out, String value, Set<Long> temporaries) {
        String masked = "(" + value + ") & " + mask(out.getSize());
        if (out.isUnique()) {
            temporaries.add(out.getOffset());
            return String.format("u_%x = %s;", out.getOffset(), masked);
        }
        if (out.isRegister() || out.getAddress().getAddressSpace().getName().equals("mem")) {
            // The value is taken whole before any byte of it is written: a place may be read in its own new value.
            StringBuilder text = new StringBuilder("{ uint64_t w = " + masked + ";");
            for (int k = 0; k < out.getSize(); k++) {
                String byteOf = k == 0 ? "w" : "w >> " + (8 * k);
                String cell = cell(out, k, false);
                text.append(cell != null ? String.format(" %s = (uint8_t)(%s);", cell, byteOf) : String.format(" arclang_mem_wr1(0x%xu, %s);", out.getOffset() + k, byteOf));
            }
            return text.append(" }").toString();
        }
        return "arclang_uncovered = true; /* a write to the flash */";
    }

    private static String signed(String value, int size) {
        return String.format("arclang_signed(%s, %d)", value, size);
    }

    /** Where a jump goes: a place of this function, another function (it does not come back), or nowhere known. */
    private String jump(Address to, AddressSetView body) {
        if (body.contains(to) && currentProgram.getListing().getInstructionAt(to) != null) {
            return "goto L_" + hex(to) + ";";
        }
        if (currentProgram.getFunctionManager().getFunctionAt(to) != null) {
            calls.add(hex(to));
            site(to);
            return "{ c_" + hex(to) + "(); return; }";
        }
        return "{ arclang_uncovered = true; return; } /* a jump to 0x" + hex(to) + ", which is no place of this function and no function */";
    }

    /** A call (or a jump that does not come back) to a function, and the registers known to hold a constant there. */
    private void site(Address to) {
        sites.append("//@ site 0x").append(place).append(" 0x").append(hex(to));
        for (long register = 18; register <= 25; register++) {
            if (constants.containsKey(register)) {
                sites.append(String.format(" r%d=0x%x", register, constants.get(register)));
            }
        }
        sites.append("\n");
    }

    /** Follow the constants an instruction loads in the registers: enough to read the arguments of a call. */
    private void follow(PcodeOp[] ops) {
        Map<Long, Long> temporaries = new HashMap<>();
        for (PcodeOp op : ops) {
            Varnode out = op.getOutput();
            if (out == null) {
                continue;
            }
            Long value = null;
            if ((op.getOpcode() == PcodeOp.COPY || op.getOpcode() == PcodeOp.INT_ZEXT) && out.getSize() == 1) {
                Varnode in = op.getInput(0);
                if (in.isConstant()) {
                    value = in.getOffset() & 0xff;
                } else if (in.isUnique()) {
                    value = temporaries.get(in.getOffset());
                } else if (isRegisterCell(in) && in.getSize() == 1) {
                    value = constants.get(in.getOffset());
                }
            }
            if (out.isUnique()) {
                if (value != null) {
                    temporaries.put(out.getOffset(), value);
                } else {
                    temporaries.remove(out.getOffset());
                }
            } else if (isRegisterCell(out)) {
                for (long k = 0; k < out.getSize(); k++) {
                    constants.remove(out.getOffset() + k);
                }
                if (value != null) {
                    constants.put(out.getOffset(), value);
                }
            }
        }
    }

    /**
     * The cycle a cut is the head of: from the cut to the last jump back to it. What is said of it:
     * whether it calls, and the registers r0..r31 its own instructions write (a register it does
     * not write, and that a function must keep, holds at the cut what the start left in it).
     */
    private String region(List<Instruction> instructions, String entry) {
        Address last = cut;
        for (Instruction instruction : instructions) {
            for (PcodeOp op : instruction.getPcode()) {
                boolean jump = op.getOpcode() == PcodeOp.BRANCH || op.getOpcode() == PcodeOp.CBRANCH;
                if (jump && op.getInput(0).isAddress() && op.getInput(0).getAddress().equals(cut) && instruction.getAddress().compareTo(last) > 0) {
                    last = instruction.getAddress();
                }
            }
        }
        Set<Long> written = new TreeSet<>();
        boolean calls = false;
        for (Instruction instruction : instructions) {
            if (instruction.getAddress().compareTo(cut) < 0 || instruction.getAddress().compareTo(last) > 0) {
                continue;
            }
            for (PcodeOp op : instruction.getPcode()) {
                calls |= op.getOpcode() == PcodeOp.CALL || op.getOpcode() == PcodeOp.CALLIND;
                if (op.getOutput() != null && isRegisterCell(op.getOutput())) {
                    for (long k = 0; k < op.getOutput().getSize(); k++) {
                        written.add(op.getOutput().getOffset() + k);
                    }
                }
            }
        }
        StringBuilder said = new StringBuilder("//@ cut 0x" + hex(cut) + " in 0x" + entry + " to 0x" + hex(last) + " calls=" + (calls ? 1 : 0) + " writes");
        for (long register : written) {
            said.append(" ").append(register);
        }
        return said.append("\n").toString();
    }

    private static boolean isRegisterCell(Varnode v) {
        return v.isAddress() && v.getAddress().getAddressSpace().getName().equals("mem") && v.getOffset() < 32;
    }

    private static boolean isMemory(Varnode v) {
        return v != null && v.isAddress() && v.getAddress().getAddressSpace().getName().equals("mem") && v.getOffset() >= 32;
    }

    private void translate(Function function, StringBuilder bodies, StringBuilder facts) {
        AddressSetView body = function.getBody();
        Set<Long> temporaries = new TreeSet<>();
        calls = new TreeSet<>();
        sites = new StringBuilder();
        constants.clear();
        Set<Long> reads = new TreeSet<>();
        Set<Long> writes = new TreeSet<>();
        int flash = 0;
        int loads = 0;
        int stores = 0;
        boolean masks = false;
        StringBuilder text = new StringBuilder();
        List<Instruction> instructions = new ArrayList<>();
        for (Instruction instruction : currentProgram.getListing().getInstructions(body, true)) {
            instructions.add(instruction);
        }
        // Where a jump arrives, what the registers hold is no longer known.
        Set<Address> arrivals = new HashSet<>();
        for (Instruction instruction : instructions) {
            for (PcodeOp op : instruction.getPcode()) {
                if ((op.getOpcode() == PcodeOp.BRANCH || op.getOpcode() == PcodeOp.CBRANCH) && op.getInput(0).isAddress()) {
                    Address to = op.getInput(0).getAddress();
                    arrivals.add(to);
                    if (body.contains(to) && to.compareTo(instruction.getAddress()) <= 0) {
                        sites.append("//@ back 0x").append(hex(instruction.getAddress())).append(" 0x").append(hex(to)).append("\n");
                    }
                }
            }
        }
        for (int rank = 0; rank < instructions.size(); rank++) {
            Instruction instruction = instructions.get(rank);
            String at = hex(instruction.getAddress());
            place = at;
            if (arrivals.contains(instruction.getAddress())) {
                constants.clear();
            }
            masks |= instruction.getMnemonicString().equals("cli");
            for (PcodeOp op : instruction.getPcode()) {
                for (Varnode in : op.getInputs()) {
                    if (isMemory(in)) {
                        for (long k = 0; k < in.getSize(); k++) {
                            reads.add(in.getOffset() + k);
                        }
                    }
                }
                if (isMemory(op.getOutput())) {
                    for (long k = 0; k < op.getOutput().getSize(); k++) {
                        writes.add(op.getOutput().getOffset() + k);
                    }
                }
                // What goes through a pointer, the stack apart (a push, a pop, a return address).
                boolean load = op.getOpcode() == PcodeOp.LOAD;
                boolean store = op.getOpcode() == PcodeOp.STORE;
                if ((load || store) && !op.getInput(1).isRegister()) {
                    boolean memory = currentProgram.getAddressFactory().getAddressSpace((int) op.getInput(0).getOffset()).getName().equals("mem");
                    if (load && !memory) {
                        flash++;
                    } else if (load) {
                        loads++;
                    } else {
                        stores++;
                    }
                }
            }
            text.append("L_").append(at).append(": /* ").append(instruction.toString().replace("*/", "* /")).append(" */\n");
            if (instruction.getAddress().equals(cut)) {
                text.append("    if (arclang_cut()) { return; }\nR_").append(at).append(": ;\n");
            }
            PcodeOp[] ops = instruction.getPcode();
            boolean inner = false;
            for (PcodeOp op : ops) {
                int opcode = op.getOpcode();
                inner |= (opcode == PcodeOp.BRANCH || opcode == PcodeOp.CBRANCH) && op.getInput(0).isConstant();
            }
            for (int k = 0; k < ops.length; k++) {
                if (inner) {
                    text.append("L_").append(at).append("_").append(k).append(": ;\n");
                }
                text.append("    ").append(operation(ops[k], k, at, body, temporaries)).append("\n");
            }
            if (inner) {
                text.append("L_").append(at).append("_").append(ops.length).append(": ;\n");
            }
            follow(ops);
            for (PcodeOp op : ops) {
                if (op.getOpcode() == PcodeOp.CALL || op.getOpcode() == PcodeOp.CALLIND) {
                    constants.clear();
                }
            }
            // Where the instruction goes on to, when that is not the one written next.
            Address next = instruction.getFallThrough();
            Address following = rank + 1 < instructions.size() ? instructions.get(rank + 1).getAddress() : null;
            if (next != null && !next.equals(following)) {
                text.append("    ").append(jump(next, body)).append("\n");
            }
        }
        String entry = hex(function.getEntryPoint());
        facts.append("//@ function 0x").append(entry).append(" calls");
        for (String callee : calls) {
            facts.append(" 0x").append(callee);
        }
        facts.append("\n");
        facts.append(sites);
        facts.append("//@ shape 0x").append(entry).append(" flash=").append(flash / 2).append(" loads=").append(loads).append(" stores=").append(stores).append(" masks=").append(masks ? 1 : 0).append("\n");
        facts.append("//@ reads 0x").append(entry);
        for (long address : reads) {
            facts.append(String.format(" 0x%x", address));
        }
        facts.append("\n//@ writes 0x").append(entry);
        for (long address : writes) {
            facts.append(String.format(" 0x%x", address));
        }
        facts.append("\n");
        bodies.append("\nvoid f_").append(entry).append("(void)\n{\n");
        for (long temporary : temporaries) {
            bodies.append(String.format("    uint64_t u_%x = 0;\n", temporary));
        }
        if (cut != null && body.contains(cut)) {
            bodies.append("    if (arclang_resume) { arclang_resume = false; goto R_").append(hex(cut)).append("; }\n");
            facts.append(region(instructions, entry));
        }
        // The function starts at its entry, which need not be its lowest address.
        bodies.append("    goto L_").append(entry).append(";\n");
        bodies.append(text);
        bodies.append("    arclang_uncovered = true; /* the function runs off its end */\n}\n");
    }

    private String operation(PcodeOp op, int rank, String at, AddressSetView body, Set<Long> temporaries) {
        Varnode out = op.getOutput();
        int size = out != null ? out.getSize() : 0;
        String a = op.getNumInputs() > 0 ? read(op.getInput(0)) : "";
        String b = op.getNumInputs() > 1 ? read(op.getInput(1)) : "";
        int sizeA = op.getNumInputs() > 0 ? op.getInput(0).getSize() : 0;
        int bits = 8 * sizeA;
        switch (op.getOpcode()) {
            case PcodeOp.COPY:
            case PcodeOp.INT_ZEXT:
                return write(out, a, temporaries);
            case PcodeOp.INT_SEXT:
                return write(out, "(uint64_t)" + signed(a, sizeA), temporaries);
            case PcodeOp.INT_ADD:
                return write(out, a + " + " + b, temporaries);
            case PcodeOp.INT_SUB:
                return write(out, a + " - " + b, temporaries);
            case PcodeOp.INT_MULT:
                return write(out, a + " * " + b, temporaries);
            case PcodeOp.INT_AND:
                return write(out, a + " & " + b, temporaries);
            case PcodeOp.INT_OR:
                return write(out, a + " | " + b, temporaries);
            case PcodeOp.INT_XOR:
                return write(out, a + " ^ " + b, temporaries);
            case PcodeOp.INT_NEGATE:
                return write(out, "~" + a, temporaries);
            case PcodeOp.INT_2COMP:
                return write(out, "UINT64_C(0) - " + a, temporaries);
            case PcodeOp.INT_EQUAL:
                return write(out, "(uint64_t)(" + a + " == " + b + ")", temporaries);
            case PcodeOp.INT_NOTEQUAL:
                return write(out, "(uint64_t)(" + a + " != " + b + ")", temporaries);
            case PcodeOp.INT_LESS:
                return write(out, "(uint64_t)(" + a + " < " + b + ")", temporaries);
            case PcodeOp.INT_LESSEQUAL:
                return write(out, "(uint64_t)(" + a + " <= " + b + ")", temporaries);
            case PcodeOp.INT_SLESS:
                return write(out, "(uint64_t)(" + signed(a, sizeA) + " < " + signed(b, sizeA) + ")", temporaries);
            case PcodeOp.INT_SLESSEQUAL:
                return write(out, "(uint64_t)(" + signed(a, sizeA) + " <= " + signed(b, sizeA) + ")", temporaries);
            case PcodeOp.INT_CARRY:
                return write(out, "(uint64_t)(((" + a + " + " + b + ") & " + mask(sizeA) + ") < " + a + ")", temporaries);
            case PcodeOp.INT_SCARRY:
                return write(out, "((~(" + a + " ^ " + b + ") & (" + a + " ^ (" + a + " + " + b + "))) >> " + (bits - 1) + ") & UINT64_C(1)", temporaries);
            case PcodeOp.INT_SBORROW:
                return write(out, "(((" + a + " ^ " + b + ") & (" + a + " ^ (" + a + " - " + b + "))) >> " + (bits - 1) + ") & UINT64_C(1)", temporaries);
            case PcodeOp.INT_LEFT:
                return write(out, "(" + b + " >= " + bits + " ? UINT64_C(0) : " + a + " << " + b + ")", temporaries);
            case PcodeOp.INT_RIGHT:
                return write(out, "(" + b + " >= " + bits + " ? UINT64_C(0) : " + a + " >> " + b + ")", temporaries);
            case PcodeOp.INT_SRIGHT:
                return write(out, "(uint64_t)(" + signed(a, sizeA) + " >> (" + b + " >= 63 ? UINT64_C(63) : " + b + "))", temporaries);
            case PcodeOp.BOOL_NEGATE:
                return write(out, "(uint64_t)(" + a + " == 0)", temporaries);
            case PcodeOp.BOOL_AND:
                return write(out, "(uint64_t)((" + a + " != 0) && (" + b + " != 0))", temporaries);
            case PcodeOp.BOOL_OR:
                return write(out, "(uint64_t)((" + a + " != 0) || (" + b + " != 0))", temporaries);
            case PcodeOp.BOOL_XOR:
                return write(out, "(uint64_t)((" + a + " != 0) != (" + b + " != 0))", temporaries);
            case PcodeOp.SUBPIECE:
                return write(out, "(" + b + " >= 8 ? UINT64_C(0) : " + a + " >> (8 * " + b + "))", temporaries);
            case PcodeOp.PIECE:
                return write(out, "(" + a + " << " + (8 * op.getInput(1).getSize()) + ") | " + b, temporaries);
            case PcodeOp.POPCOUNT:
                return write(out, "arclang_popcount(" + a + ")", temporaries);
            case PcodeOp.LOAD: {
                String space = currentProgram.getAddressFactory().getAddressSpace((int) op.getInput(0).getOffset()).getName();
                String pointer = read(op.getInput(1));
                if (space.equals("mem")) {
                    return write(out, "arclang_mem_rd(" + pointer + ", " + size + ")", temporaries);
                }
                // The flash, by word (the space of the code) or by byte.
                int unit = currentProgram.getAddressFactory().getAddressSpace((int) op.getInput(0).getOffset()).getAddressableUnitSize();
                return write(out, "arclang_flash_rd(" + pointer + " * " + unit + ", " + size + ")", temporaries);
            }
            case PcodeOp.STORE: {
                String space = currentProgram.getAddressFactory().getAddressSpace((int) op.getInput(0).getOffset()).getName();
                if (!space.equals("mem")) {
                    return "arclang_uncovered = true; /* a write to the flash */";
                }
                Varnode value = op.getInput(2);
                return "arclang_mem_wr(" + read(op.getInput(1)) + ", " + value.getSize() + ", " + read(value) + ");";
            }
            case PcodeOp.BRANCH:
                return target(op, rank, at, body);
            case PcodeOp.CBRANCH:
                return "if (" + b + " != 0) " + target(op, rank, at, body);
            case PcodeOp.CALL: {
                Address to = op.getInput(0).getAddress();
                if (currentProgram.getFunctionManager().getFunctionAt(to) == null) {
                    return "{ arclang_uncovered = true; return; } /* a call to 0x" + hex(to) + ", where no function was found */";
                }
                calls.add(hex(to));
                site(to);
                return "c_" + hex(to) + "(); if (arclang_stop) { return; }";
            }
            case PcodeOp.RETURN:
                return "return;";
            case PcodeOp.CALLOTHER: {
                String name = currentProgram.getLanguage().getUserDefinedOpName((int) op.getInput(0).getOffset());
                userOps.add(name);
                return "arclang_op_" + name + "();";
            }
            case PcodeOp.BRANCHIND:
            case PcodeOp.CALLIND:
                return "{ arclang_uncovered = true; return; } /* a computed jump */";
            default:
                return "{ arclang_uncovered = true; return; } /* " + op.getMnemonic() + ": not translated */";
        }
    }

    /** The target of a branch: an address, or another operation of the same instruction. */
    private String target(PcodeOp op, int rank, String at, AddressSetView body) {
        Varnode to = op.getInput(0);
        if (to.isConstant()) {
            return "goto L_" + at + "_" + (rank + (int) to.getOffset()) + ";";
        }
        return jump(to.getAddress(), body);
    }
}
