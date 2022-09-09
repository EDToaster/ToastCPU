
#[derive(Debug)]
pub enum PushPopOpcode {
    PUSH, POP,
}

#[derive(Debug)]
pub enum LoadStoreOpcode {
    LOAD, STR, 
}

#[derive(Debug)]
pub enum IMoveOpcode {
    IMOV, IMOH,
}

#[derive(Debug)]
pub enum NoArgumentOpcode {
    RTI, HALT
}

#[derive(Debug)]
pub enum JumpOpcode {
    JMP, JZ, JNZ, JN, JP,
}

#[derive(Debug)]
pub enum JumpLinkOpcode {
    JMPL, JZL, JNZL, JNL, JPL,
}

#[derive(Debug)]
pub enum JumpRetOpcode {
    JMPR, JZR, JNZR, JNR, JPR,
}

#[derive(Debug)]
pub enum SingleALUOpcode {
    NOT,
}

#[derive(Debug)]
pub enum ALUOpcode {
    AND, OR, XOR, ADD, SUB, MOV, SHR, SSHR, SHL,
}

#[derive(Debug)]
pub enum IALUOpcode {
    IAND, IOR, IXOR, IADD, ISUB, ISHR, ISSHR, ISHL,
}