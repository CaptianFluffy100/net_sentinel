# Adding Monaco Editor Language Server for Pseudo-Code

This guide explains how to add a code server (language server) for Monaco Editor to provide syntax highlighting, autocomplete, suggestions, error checking, and warnings for the pseudo-code language used in Net Sentinel.

## Overview

Monaco Editor supports custom language definitions through:
- **Monarch Tokenizer**: For syntax highlighting and tokenization
- **Completion Item Provider**: For autocomplete and suggestions
- **Hover Provider**: For hover documentation (optional)
- **Document Symbol Provider**: For outline/navigation (optional)

## Step 1: Register the Custom Language

First, register a new language ID for your pseudo-code. Add this code **before** initializing the Monaco Editor:

```javascript
// Register the pseudo-code language
monaco.languages.register({ id: 'pseudo-code' });
```

## Step 2: Define Syntax Highlighting (Monarch Tokenizer)

Create a Monarch tokenizer to define syntax highlighting rules. This tells Monaco how to color different parts of your code:

```javascript
// Set Monarch tokenizer for syntax highlighting
monaco.languages.setMonarchTokensProvider('pseudo-code', {
    // Keywords - commands that start blocks or control flow
    keywords: [
        'PACKET_START', 'PACKET_END',
        'RESPONSE_START', 'RESPONSE_END',
        'CODE_START', 'CODE_END',
        'OUTPUT_SUCCESS', 'OUTPUT_ERROR', 'OUTPUT_END',
        'IF', 'ELSE', 'ELSE IF', 'FOR', 'IN', 'BREAK',
        'RETURN', 'JSON_OUTPUT', 'CONNECTION_CLOSE'
    ],
    
    // WRITE commands
    writeCommands: [
        'WRITE_BYTE', 'WRITE_SHORT', 'WRITE_SHORT_BE',
        'WRITE_INT', 'WRITE_INT_BE', 'WRITE_VARINT',
        'WRITE_STRING', 'WRITE_STRING_LEN', 'WRITE_BYTES'
    ],
    
    // READ commands
    readCommands: [
        'READ_BYTE', 'READ_SHORT', 'READ_SHORT_BE',
        'READ_INT', 'READ_INT_BE', 'READ_VARINT',
        'READ_STRING', 'READ_STRING_NULL', 'SKIP_BYTES'
    ],
    
    // Validation commands
    validationCommands: [
        'EXPECT_BYTE', 'EXPECT_MAGIC'
    ],
    
    // Variable types
    types: [
        'STRING', 'INT', 'BYTE', 'FLOAT', 'ARRAY'
    ],
    
    // Built-in functions
    functions: [
        'SPLIT', 'REPLACE', 'CONTAINS'
    ],
    
    // Special placeholders
    placeholders: [
        'PACKET_LEN', 'HOST', 'PORT', 'IP', 'IP_LEN', 'IP_LEN_HEX', 'ERROR'
    ],
    
    // Tokenizer rules
    tokenizer: {
        root: [
            // Comments - lines starting with #
            [/^#.*$/, 'comment'],
            [/[#].*$/, 'comment'],
            
            // Strings - quoted strings
            [/"/, { token: 'string.quote', bracket: '@open', next: '@string' }],
            [/'/, { token: 'string.quote', bracket: '@open', next: '@stringSingle' }],
            
            // Numbers - decimal and hexadecimal
            [/\b0[xX][0-9a-fA-F]+\b/, 'number.hex'],
            [/\b\d+\.\d+\b/, 'number.float'],
            [/\b\d+\b/, 'number'],
            
            // Keywords
            [/\b(PACKET_START|PACKET_END|RESPONSE_START|RESPONSE_END|CODE_START|CODE_END|OUTPUT_SUCCESS|OUTPUT_ERROR|OUTPUT_END)\b/, 'keyword'],
            
            // Control flow
            [/\b(IF|ELSE|FOR|IN|BREAK)\b/, 'keyword.control'],
            
            // Write commands
            [/\b(WRITE_BYTE|WRITE_SHORT|WRITE_SHORT_BE|WRITE_INT|WRITE_INT_BE|WRITE_VARINT|WRITE_STRING|WRITE_STRING_LEN|WRITE_BYTES)\b/, 'keyword.write'],
            
            // Read commands
            [/\b(READ_BYTE|READ_SHORT|READ_SHORT_BE|READ_INT|READ_INT_BE|READ_VARINT|READ_STRING|READ_STRING_NULL|SKIP_BYTES)\b/, 'keyword.read'],
            
            // Validation commands
            [/\b(EXPECT_BYTE|EXPECT_MAGIC)\b/, 'keyword.validation'],
            
            // Variable types
            [/\b(STRING|INT|BYTE|FLOAT|ARRAY)\b/, 'type'],
            
            // Functions
            [/\b(SPLIT|REPLACE|CONTAINS|JSON_OUTPUT|RETURN)\b/, 'function'],
            
            // Placeholders
            [/\b(PACKET_LEN|HOST|PORT|IP|IP_LEN|IP_LEN_HEX|ERROR)\b/, 'variable.predefined'],
            
            // Operators
            [/[=!<>]=/, 'operator'],
            [/[<>=!]/, 'operator'],
            [/[+\-*/]/, 'operator'],
            
            // Variables - identifiers
            [/[a-zA-Z_][a-zA-Z0-9_]*/, {
                cases: {
                    '@keywords': 'keyword',
                    '@writeCommands': 'keyword.write',
                    '@readCommands': 'keyword.read',
                    '@validationCommands': 'keyword.validation',
                    '@types': 'type',
                    '@functions': 'function',
                    '@placeholders': 'variable.predefined',
                    '@default': 'variable'
                }
            }],
            
            // Whitespace
            { include: '@whitespace' },
        ],
        
        // String handling
        string: [
            [/[^\\"]+/, 'string'],
            [/\\./, 'string.escape.invalid'],
            [/"/, { token: 'string.quote', bracket: '@close', next: '@pop' }]
        ],
        
        stringSingle: [
            [/[^\\']+/, 'string'],
            [/\\./, 'string.escape.invalid'],
            [/'/, { token: 'string.quote', bracket: '@close', next: '@pop' }]
        ],
        
        whitespace: [
            [/[ \t\r\n]+/, 'white'],
        ],
    },
});
```

## Step 3: Define Token Colors

Configure how tokens are colored in the editor. Add this after the tokenizer:

```javascript
// Define token colors
monaco.editor.defineTheme('pseudo-code-theme', {
    base: 'vs-dark',
    inherit: true,
    rules: [
        { token: 'comment', foreground: '6A9955', fontStyle: 'italic' },
        { token: 'keyword', foreground: '569CD6', fontStyle: 'bold' },
        { token: 'keyword.control', foreground: 'C586C0' },
        { token: 'keyword.write', foreground: '4EC9B0' },
        { token: 'keyword.read', foreground: 'DCDCAA' },
        { token: 'keyword.validation', foreground: 'CE9178' },
        { token: 'type', foreground: '4EC9B0' },
        { token: 'function', foreground: 'DCDCAA' },
        { token: 'variable', foreground: '9CDCFE' },
        { token: 'variable.predefined', foreground: '569CD6', fontStyle: 'bold' },
        { token: 'string', foreground: 'CE9178' },
        { token: 'number', foreground: 'B5CEA8' },
        { token: 'number.hex', foreground: 'B5CEA8' },
        { token: 'number.float', foreground: 'B5CEA8' },
        { token: 'operator', foreground: 'D4D4D4' },
    ],
    colors: {}
});
```

## Step 4: Add Autocomplete/Suggestions

Create a completion provider to suggest commands, keywords, and provide documentation:

```javascript
// Register completion provider for autocomplete
monaco.languages.registerCompletionItemProvider('pseudo-code', {
    provideCompletionItems: (model, position) => {
        const word = model.getWordUntilPosition(position);
        const range = {
            startLineNumber: position.lineNumber,
            endLineNumber: position.lineNumber,
            startColumn: word.startColumn,
            endColumn: word.endColumn
        };
        
        // Get current line content up to cursor
        const lineContent = model.getLineContent(position.lineNumber);
        const textUntilPosition = lineContent.substring(0, position.column - 1).trim();
        
        // Command completions
        const suggestions = [];
        
        // Packet construction commands
        suggestions.push(
            {
                label: 'PACKET_START',
                kind: monaco.languages.CompletionItemKind.Keyword,
                documentation: 'Marks the beginning of a packet definition',
                insertText: 'PACKET_START',
                range: range
            },
            {
                label: 'PACKET_END',
                kind: monaco.languages.CompletionItemKind.Keyword,
                documentation: 'Marks the end of a packet definition',
                insertText: 'PACKET_END',
                range: range
            },
            {
                label: 'WRITE_BYTE',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Writes a single byte (0-255). Example: WRITE_BYTE 0xFF',
                insertText: 'WRITE_BYTE ${1:value}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'WRITE_SHORT',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Writes a 16-bit integer (little-endian). Example: WRITE_SHORT 1234',
                insertText: 'WRITE_SHORT ${1:value}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'WRITE_SHORT_BE',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Writes a 16-bit integer (big-endian/network byte order). Example: WRITE_SHORT_BE 1234',
                insertText: 'WRITE_SHORT_BE ${1:value}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'WRITE_INT',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Writes a 32-bit integer (little-endian). Example: WRITE_INT 50000',
                insertText: 'WRITE_INT ${1:value}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'WRITE_INT_BE',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Writes a 32-bit integer (big-endian/network byte order). Example: WRITE_INT_BE PACKET_LEN',
                insertText: 'WRITE_INT_BE ${1:value}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'WRITE_VARINT',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Writes a variable-length integer (Minecraft-style). Example: WRITE_VARINT 300',
                insertText: 'WRITE_VARINT ${1:value}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'WRITE_STRING',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Writes a null-terminated string. Example: WRITE_STRING "Hello Server"',
                insertText: 'WRITE_STRING "${1:text}"',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'WRITE_STRING_LEN',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Writes a fixed-length string. Example: WRITE_STRING_LEN "Test" 10',
                insertText: 'WRITE_STRING_LEN "${1:text}" ${2:length}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'WRITE_BYTES',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Writes raw hexadecimal bytes. Example: WRITE_BYTES "FF00AA55"',
                insertText: 'WRITE_BYTES "${1:hex}"',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            }
        );
        
        // Response parsing commands
        suggestions.push(
            {
                label: 'RESPONSE_START',
                kind: monaco.languages.CompletionItemKind.Keyword,
                documentation: 'Marks the beginning of response parsing rules',
                insertText: 'RESPONSE_START',
                range: range
            },
            {
                label: 'RESPONSE_END',
                kind: monaco.languages.CompletionItemKind.Keyword,
                documentation: 'Marks the end of response parsing rules',
                insertText: 'RESPONSE_END',
                range: range
            },
            {
                label: 'READ_BYTE',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Reads a single byte and stores it in a variable. Example: READ_BYTE packet_id',
                insertText: 'READ_BYTE ${1:var_name}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'READ_SHORT',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Reads a 16-bit integer (little-endian). Example: READ_SHORT player_count',
                insertText: 'READ_SHORT ${1:var_name}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'READ_SHORT_BE',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Reads a 16-bit integer (big-endian). Example: READ_SHORT_BE port_number',
                insertText: 'READ_SHORT_BE ${1:var_name}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'READ_INT',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Reads a 32-bit integer (little-endian). Example: READ_INT server_version',
                insertText: 'READ_INT ${1:var_name}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'READ_INT_BE',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Reads a 32-bit integer (big-endian). Example: READ_INT_BE response_length',
                insertText: 'READ_INT_BE ${1:var_name}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'READ_VARINT',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Reads a variable-length integer. Example: READ_VARINT packet_length',
                insertText: 'READ_VARINT ${1:var_name}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'READ_STRING',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Reads a fixed-length string. Example: READ_STRING server_name 32',
                insertText: 'READ_STRING ${1:var_name} ${2:length}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'READ_STRING_NULL',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Reads a null-terminated string. Example: READ_STRING_NULL server_name',
                insertText: 'READ_STRING_NULL ${1:var_name}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'SKIP_BYTES',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Skips the specified number of bytes. Example: SKIP_BYTES 4',
                insertText: 'SKIP_BYTES ${1:count}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            }
        );
        
        // Validation commands
        suggestions.push(
            {
                label: 'EXPECT_BYTE',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Validates that the next byte matches the expected value. Example: EXPECT_BYTE 0xFE',
                insertText: 'EXPECT_BYTE ${1:value}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'EXPECT_MAGIC',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Validates that the next bytes match the expected magic bytes. Example: EXPECT_MAGIC "FEEDFACE"',
                insertText: 'EXPECT_MAGIC "${1:hex_string}"',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            }
        );
        
        // Code block commands
        suggestions.push(
            {
                label: 'CODE_START',
                kind: monaco.languages.CompletionItemKind.Keyword,
                documentation: 'Marks the beginning of a code block',
                insertText: 'CODE_START',
                range: range
            },
            {
                label: 'CODE_END',
                kind: monaco.languages.CompletionItemKind.Keyword,
                documentation: 'Marks the end of a code block',
                insertText: 'CODE_END',
                range: range
            }
        );
        
        // Control flow
        suggestions.push(
            {
                label: 'IF',
                kind: monaco.languages.CompletionItemKind.Keyword,
                documentation: 'Conditional execution. Example: IF condition == 1:',
                insertText: 'IF ${1:condition}:',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'ELSE',
                kind: monaco.languages.CompletionItemKind.Keyword,
                documentation: 'Else clause for IF statements',
                insertText: 'ELSE:',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'FOR',
                kind: monaco.languages.CompletionItemKind.Keyword,
                documentation: 'Loop over an array. Example: FOR item IN array:',
                insertText: 'FOR ${1:var_name} IN ${2:array_name}:',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            }
        );
        
        // Variable types
        suggestions.push(
            {
                label: 'STRING',
                kind: monaco.languages.CompletionItemKind.TypeParameter,
                documentation: 'Declare a string variable. Example: STRING name = "value"',
                insertText: 'STRING ${1:name} = ${2:"value"}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'INT',
                kind: monaco.languages.CompletionItemKind.TypeParameter,
                documentation: 'Declare an integer variable. Example: INT count = 10',
                insertText: 'INT ${1:name} = ${2:value}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'BYTE',
                kind: monaco.languages.CompletionItemKind.TypeParameter,
                documentation: 'Declare a byte variable. Example: BYTE status = 0xFF',
                insertText: 'BYTE ${1:name} = ${2:value}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'FLOAT',
                kind: monaco.languages.CompletionItemKind.TypeParameter,
                documentation: 'Declare a float variable. Example: FLOAT version = 1.19',
                insertText: 'FLOAT ${1:name} = ${2:value}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'ARRAY',
                kind: monaco.languages.CompletionItemKind.TypeParameter,
                documentation: 'Declare an array variable. Example: ARRAY items = ["a", "b", "c"]',
                insertText: 'ARRAY ${1:name} = ${2:["value1", "value2"]}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            }
        );
        
        // Functions
        suggestions.push(
            {
                label: 'SPLIT',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Splits a string by delimiter. Example: SPLIT(var_name, ",")',
                insertText: 'SPLIT(${1:var_name}, ${2:"delimiter"})',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'REPLACE',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Replaces all occurrences in a string. Example: REPLACE(var_name, "old", "new")',
                insertText: 'REPLACE(${1:var_name}, ${2:"search"}, ${3:"replace"})',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'JSON_OUTPUT',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Parses a string variable as JSON. Example: JSON_OUTPUT JSON_PAYLOAD',
                insertText: 'JSON_OUTPUT ${1:var_name}',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'RETURN',
                kind: monaco.languages.CompletionItemKind.Function,
                documentation: 'Formats the expression into Prometheus metric labels. Example: RETURN "server=HOST, protocol=1"',
                insertText: 'RETURN "${1:expression}"',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            }
        );
        
        // Output blocks
        suggestions.push(
            {
                label: 'OUTPUT_SUCCESS',
                kind: monaco.languages.CompletionItemKind.Keyword,
                documentation: 'Marks output block that executes on success',
                insertText: 'OUTPUT_SUCCESS\n  ${1:// commands}\nOUTPUT_END',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            },
            {
                label: 'OUTPUT_ERROR',
                kind: monaco.languages.CompletionItemKind.Keyword,
                documentation: 'Marks output block that executes on error',
                insertText: 'OUTPUT_ERROR\n  ${1:// commands}\nOUTPUT_END',
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                range: range
            }
        );
        
        // Special placeholders
        suggestions.push(
            {
                label: 'PACKET_LEN',
                kind: monaco.languages.CompletionItemKind.Constant,
                documentation: 'Auto-calculated packet length placeholder',
                insertText: 'PACKET_LEN',
                range: range
            },
            {
                label: 'HOST',
                kind: monaco.languages.CompletionItemKind.Constant,
                documentation: 'Server hostname/address placeholder',
                insertText: 'HOST',
                range: range
            },
            {
                label: 'PORT',
                kind: monaco.languages.CompletionItemKind.Constant,
                documentation: 'Server port number placeholder',
                insertText: 'PORT',
                range: range
            },
            {
                label: 'IP',
                kind: monaco.languages.CompletionItemKind.Constant,
                documentation: 'Server IP address placeholder',
                insertText: 'IP',
                range: range
            }
        );
        
        return { suggestions: suggestions };
    }
});
```

## Step 5: Add Error Checking and Warnings (Optional)

For advanced validation, you can add a diagnostic provider. This requires parsing the code and checking for errors:

```javascript
// Register diagnostic provider for errors/warnings
monaco.languages.registerDocumentSymbolProvider('pseudo-code', {
    provideDocumentSymbols: (model) => {
        const symbols = [];
        const lines = model.getLinesContent();
        
        // Track block structure for validation
        let packetStartCount = 0;
        let packetEndCount = 0;
        let responseStartCount = 0;
        let responseEndCount = 0;
        let codeStartCount = 0;
        let codeEndCount = 0;
        let outputSuccessCount = 0;
        let outputErrorCount = 0;
        let outputEndCount = 0;
        
        lines.forEach((line, index) => {
            const lineNumber = index + 1;
            const trimmed = line.trim();
            
            // Count blocks
            if (trimmed === 'PACKET_START') packetStartCount++;
            if (trimmed === 'PACKET_END') packetEndCount++;
            if (trimmed === 'RESPONSE_START') responseStartCount++;
            if (trimmed === 'RESPONSE_END') responseEndCount++;
            if (trimmed === 'CODE_START') codeStartCount++;
            if (trimmed === 'CODE_END') codeEndCount++;
            if (trimmed === 'OUTPUT_SUCCESS') outputSuccessCount++;
            if (trimmed === 'OUTPUT_ERROR') outputErrorCount++;
            if (trimmed === 'OUTPUT_END') outputEndCount++;
        });
        
        // This is a simple example - you could return markers here
        // For full validation, use markdown markers API
        return symbols;
    }
});

// Add markers for validation errors (call this when code changes)
function validatePseudoCode(editor) {
    const model = editor.getModel();
    const markers = [];
    const lines = model.getLinesContent();
    
    // Track block structure
    const blockStack = [];
    
    lines.forEach((line, index) => {
        const lineNumber = index + 1;
        const trimmed = line.trim();
        
        // Skip comments and empty lines
        if (trimmed.startsWith('#') || trimmed === '') return;
        
        // Check for unmatched blocks
        if (trimmed === 'PACKET_END' && blockStack[blockStack.length - 1] !== 'PACKET_START') {
            markers.push({
                severity: monaco.MarkerSeverity.Error,
                startLineNumber: lineNumber,
                startColumn: 1,
                endLineNumber: lineNumber,
                endColumn: trimmed.length + 1,
                message: 'PACKET_END without matching PACKET_START'
            });
        } else if (trimmed === 'PACKET_START') {
            blockStack.push('PACKET_START');
        } else if (trimmed === 'PACKET_END') {
            blockStack.pop();
        }
        
        // Similar checks for other block types...
        // (This is simplified - full implementation would track all block types)
    });
    
    // Check for unclosed blocks
    if (blockStack.length > 0) {
        markers.push({
            severity: monaco.MarkerSeverity.Warning,
            startLineNumber: 1,
            startColumn: 1,
            endLineNumber: 1,
            endColumn: 1,
            message: `Unclosed blocks detected: ${blockStack.join(', ')}`
        });
    }
    
    // Apply markers to the editor
    monaco.editor.setModelMarkers(model, 'pseudo-code-validator', markers);
}

// Call validation when content changes
// (Add this to your editor initialization)
editor.onDidChangeModelContent(() => {
    validatePseudoCode(editor);
});
```

## Step 6: Update Editor Configuration

Finally, update your Monaco Editor initialization to use the new language:

```javascript
// Initialize Monaco Editor
require.config({ paths: { vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.45.0/min/vs' } });
require(['vs/editor/editor.main'], function () {
    // Register language and configure (all the code from steps 1-5 goes here)
    
    // Create editor with the new language
    monacoEditor = monaco.editor.create(document.getElementById('monaco-editor-container'), {
        value: `# Minecraft-style query
PACKET_START
WRITE_BYTE 0xFE
WRITE_BYTE 0xFD
WRITE_BYTE 0x09
WRITE_INT 0x00000000
PACKET_END

RESPONSE_START
EXPECT_BYTE 0xFE
EXPECT_BYTE 0xFD
READ_BYTE packet_type
READ_STRING_NULL session_id
READ_STRING_NULL challenge_token
RESPONSE_END`,
        language: 'pseudo-code',  // Changed from 'plaintext' to 'pseudo-code'
        theme: 'pseudo-code-theme',  // Use custom theme
        automaticLayout: true,
        fontSize: 14,
        minimap: { enabled: true },
        wordWrap: 'on',
        lineNumbers: 'on',
        scrollBeyondLastLine: false,
        suggestOnTriggerCharacters: true,
        quickSuggestions: {
            other: true,
            comments: false,
            strings: false
        },
        parameterHints: {
            enabled: true
        },
    });
    
    // Add validation on content change
    monacoEditor.onDidChangeModelContent(() => {
        validatePseudoCode(monacoEditor);
    });
});
```

## Complete Example Integration

Here's a complete example showing where to place all the code in your `index.html`:

```javascript
<script>
    const VERSION = '{{VERSION}}';
    let monacoEditor = null;

    // Initialize Monaco Editor
    require.config({ paths: { vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.45.0/min/vs' } });
    require(['vs/editor/editor.main'], function () {
        // ===== STEP 1: Register Language =====
        monaco.languages.register({ id: 'pseudo-code' });
        
        // ===== STEP 2: Define Syntax Highlighting =====
        // (Monarch tokenizer code from Step 2)
        
        // ===== STEP 3: Define Token Colors =====
        // (Theme definition code from Step 3)
        
        // ===== STEP 4: Add Autocomplete =====
        // (Completion provider code from Step 4)
        
        // ===== STEP 5: Add Error Checking =====
        // (Validation code from Step 5 - optional)
        
        // ===== STEP 6: Create Editor =====
        monacoEditor = monaco.editor.create(document.getElementById('monaco-editor-container'), {
            value: `# Minecraft-style query
PACKET_START
WRITE_BYTE 0xFE
WRITE_BYTE 0xFD
WRITE_BYTE 0x09
WRITE_INT 0x00000000
PACKET_END

RESPONSE_START
EXPECT_BYTE 0xFE
EXPECT_BYTE 0xFD
READ_BYTE packet_type
READ_STRING_NULL session_id
READ_STRING_NULL challenge_token
RESPONSE_END`,
            language: 'pseudo-code',
            theme: 'pseudo-code-theme',
            automaticLayout: true,
            fontSize: 14,
            minimap: { enabled: true },
            wordWrap: 'on',
            lineNumbers: 'on',
            scrollBeyondLastLine: false,
            suggestOnTriggerCharacters: true,
            quickSuggestions: {
                other: true,
                comments: false,
                strings: false
            },
        });
        
        // Add validation when content changes
        if (typeof validatePseudoCode === 'function') {
            monacoEditor.onDidChangeModelContent(() => {
                validatePseudoCode(monacoEditor);
            });
        }
    });
    
    // Rest of your existing JavaScript code...
</script>
```

## Tips and Best Practices

1. **Performance**: The tokenizer and completion provider run on every keystroke. Keep them efficient.

2. **Snippets**: Use snippet syntax (`${1:placeholder}`) in completion items to create placeholders that users can tab through.

3. **Documentation**: Include helpful documentation strings in completion items - they appear in the hover tooltip.

4. **Validation**: For complex validation, consider doing it asynchronously or debouncing to avoid blocking the UI.

5. **Testing**: Test your language server with various code patterns from your documentation to ensure all commands are properly highlighted and suggested.

6. **Extending**: You can add more providers like:
   - `registerHoverProvider`: Show documentation on hover
   - `registerFoldingRangeProvider`: Enable code folding
   - `registerSignatureHelpProvider`: Show function signatures

## Resources

- [Monaco Editor Language API Documentation](https://microsoft.github.io/monaco-editor/playground.html#extending-language-services-custom-languages)
- [Monarch Tokenizer Documentation](https://microsoft.github.io/monaco-editor/monarch.html)
- [VS Code Language Extension Guide](https://code.visualstudio.com/api/language-extensions/overview) (helpful for understanding concepts)

