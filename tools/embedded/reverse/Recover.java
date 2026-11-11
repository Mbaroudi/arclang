// What Ghidra finds in a firmware binary, written as plain text: every function with its
// address range, the functions it calls, the registers of the microcontroller it reads and
// writes, and its decompiled C. Run headless:
//
//   analyzeHeadless <project dir> <name> -import <file> -processor <language> ... \
//       -scriptPath /opt/arclang -postScript Recover.java <out.txt>
//
//@category ArcLang
import java.io.PrintWriter;
import java.util.Set;
import java.util.TreeSet;

import ghidra.app.decompiler.DecompInterface;
import ghidra.app.decompiler.DecompileResults;
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.address.AddressSpace;
import ghidra.program.model.listing.Function;
import ghidra.program.model.listing.Instruction;
import ghidra.program.model.symbol.Reference;

public class Recover extends GhidraScript {
    private static final int DECOMPILE_SECONDS = 120;

    @Override
    public void run() throws Exception {
        String[] args = getScriptArgs();
        if (args.length != 1) {
            throw new IllegalArgumentException("usage: Recover.java <output file>");
        }
        startAtTheFirstByte();
        DecompInterface decompiler = new DecompInterface();
        decompiler.openProgram(currentProgram);
        try (PrintWriter out = new PrintWriter(args[0], "UTF-8")) {
            out.println("language " + currentProgram.getLanguageID()
                + " compiler " + currentProgram.getCompilerSpec().getCompilerSpecID());
            for (Function function : currentProgram.getFunctionManager().getFunctions(true)) {
                describe(function, decompiler, out);
            }
        } finally {
            decompiler.dispose();
        }
    }

    /**
     * A raw image names no entry point, so the analysis finds nothing to follow. A firmware
     * image starts executing at its first byte: start there and analyse again.
     */
    private void startAtTheFirstByte() throws Exception {
        if (currentProgram.getFunctionManager().getFunctionCount() > 0) {
            return;
        }
        Address first = currentProgram.getMinAddress();
        disassemble(first);
        createFunction(first, "reset");
        analyzeAll(currentProgram);
    }

    private void describe(Function function, DecompInterface decompiler, PrintWriter out) {
        out.println();
        out.println("== function " + function.getName() + " at " + function.getEntryPoint()
            + " body " + function.getBody());
        Set<String> calls = new TreeSet<>();
        for (Function callee : function.getCalledFunctions(monitor)) {
            calls.add(callee.getName());
        }
        out.println("calls " + calls);
        Set<String> reads = new TreeSet<>();
        Set<String> writes = new TreeSet<>();
        for (Instruction instruction : currentProgram.getListing().getInstructions(function.getBody(), true)) {
            for (Reference reference : instruction.getReferencesFrom()) {
                Address to = reference.getToAddress();
                if (!to.isMemoryAddress() || isCode(to.getAddressSpace())) {
                    continue;
                }
                if (reference.getReferenceType().isWrite()) {
                    writes.add(to.toString());
                }
                if (reference.getReferenceType().isRead()) {
                    reads.add(to.toString());
                }
            }
        }
        out.println("reads " + reads);
        out.println("writes " + writes);
        DecompileResults results = decompiler.decompileFunction(function, DECOMPILE_SECONDS, monitor);
        if (results.decompileCompleted()) {
            out.println(results.getDecompiledFunction().getC());
        } else {
            out.println("-- not decompiled: " + results.getErrorMessage());
        }
    }

    private boolean isCode(AddressSpace space) {
        return space.equals(currentProgram.getAddressFactory().getDefaultAddressSpace());
    }
}
