# RFC 0004: Distributed Systems Grammar Specification

- **Status:** Ratified
- **Author:** UEAS Automated Agent
- **Date:** 2026-07-10
- **Supersedes:** None
- **Superseded By:** None

## Motivation

ADR 0009 establishes the architecture for simulating distributed algorithms within the UEAS abstract interpreter's Virtual Heap. To support this, we must extend the UEAS grammar (`UEAS.g4`) to allow specifying message-passing semantics between virtual nodes.

## Proposed Change

### 1. New Primitive Type
Introduce a `Node` primitive type representing an actor in the distributed simulation.

### 2. Message Passing Statements
Introduce explicit syntax for sending messages and handling asynchronous receipts.

```ueas
Algorithm PaxosPropose(value)
    Require: value: Integer
    Ensure: Void
    Complexity: "Message: O(N), Round: O(1)"

    for each n in cluster do
        send { type: "PROPOSE", val: value } to n
    end for

    on receive msg from n do
        if msg.type == "PROMISE" then
            # Handle consensus logic
        end if
    end on
```

### 3. Grammar Updates

**Lexer:**
```ebnf
SEND    ::= 'send' | 'Send' | 'SEND'
TO      ::= 'to' | 'To' | 'TO'
ON      ::= 'on' | 'On' | 'ON'
RECEIVE ::= 'receive' | 'Receive' | 'RECEIVE'
FROM    ::= 'from' | 'From' | 'FROM'
NODE    ::= 'Node'
```

**Parser:**
```ebnf
statement ::= ...
            | sendStmt NEWLINE
            | onReceiveBlock

sendStmt  ::= 'send' expression 'to' expression

onReceiveBlock ::= 'on' 'receive' IDENTIFIER 'from' IDENTIFIER 'do' NEWLINE
                   block
                   'end' 'on' NEWLINE
```

### 4. AST Schema Updates
- **SendStatement**: `{ "kind": "SendStatement", "message": Expression, "recipient": Expression }`
- **OnReceiveBlock**: `{ "kind": "OnReceiveBlock", "messageVar": string, "senderVar": string, "body": Statement[] }`

## Impact Analysis
- **Backward Compatibility:** Fully backward compatible.
- **Kernel:** The kernel must implement an event loop for the Virtual Network Topology to schedule `on receive` blocks deterministically.
