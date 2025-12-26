// Monaco Editor Language Server for Pseudo-Code
// This file is embedded in the Rust binary and served to the client

(function() {
    'use strict';
    
    // Wait for Monaco to be loaded (it's already loaded from CDN in the HTML)
    function initLanguageServer() {
        // Check if Monaco is already loaded
        if (typeof monaco !== 'undefined' && typeof monaco.languages !== 'undefined') {
            registerLanguageServer();
        } else {
            // If Monaco isn't loaded yet, wait a bit and try again
            setTimeout(initLanguageServer, 50);
        }
    }
    
    function registerLanguageServer() {
        // Prevent duplicate registration
        if (window.pseudoCodeLanguageServerRegistered) {
            return;
        }
        window.pseudoCodeLanguageServerRegistered = true;
        
        // ===== STEP 1: Register Language =====
        monaco.languages.register({ id: 'pseudo-code' });
        
        // ===== STEP 2: Define Syntax Highlighting (Monarch Tokenizer) =====
        monaco.languages.setMonarchTokensProvider('pseudo-code', {
            keywords: [
                'PACKET_START', 'PACKET_END',
                'RESPONSE_START', 'RESPONSE_END',
                'CODE_START', 'CODE_END',
                'OUTPUT_SUCCESS', 'OUTPUT_ERROR', 'OUTPUT_END',
                'IF', 'ELSE', 'FOR', 'IN', 'BREAK',
                'RETURN', 'JSON_OUTPUT', 'CONNECTION_CLOSE'
            ],
            
            writeCommands: [
                'WRITE_BYTE', 'WRITE_SHORT', 'WRITE_SHORT_BE',
                'WRITE_INT', 'WRITE_INT_BE', 'WRITE_VARINT',
                'WRITE_STRING', 'WRITE_STRING_LEN', 'WRITE_BYTES'
            ],
            
            readCommands: [
                'READ_BYTE', 'READ_SHORT', 'READ_SHORT_BE',
                'READ_INT', 'READ_INT_BE', 'READ_VARINT',
                'READ_STRING', 'READ_STRING_NULL', 'SKIP_BYTES'
            ],
            
            validationCommands: [
                'EXPECT_BYTE', 'EXPECT_MAGIC'
            ],
            
            types: [
                'STRING', 'INT', 'BYTE', 'FLOAT', 'ARRAY'
            ],
            
            functions: [
                'SPLIT', 'REPLACE', 'CONTAINS'
            ],
            
            placeholders: [
                'PACKET_LEN', 'HOST', 'PORT', 'IP', 'IP_LEN', 'IP_LEN_HEX', 'ERROR'
            ],
            
            tokenizer: {
                root: [
                    // Comments - lines starting with # or inline comments
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
                    [/\b(SPLIT|REPLACE|CONTAINS|JSON_OUTPUT)\b/, 'function'],
                    // RETURN gets special styling
                    [/\bRETURN\b/, 'function.return'],
                    
                    // Placeholders
                    [/\b(PACKET_LEN|HOST|PORT|IP|IP_LEN|IP_LEN_HEX|ERROR)\b/, 'variable.predefined'],
                    
                    // Operators
                    [/[=!<>]=/, 'operator'],
                    [/[<>=!]/, 'operator'],
                    [/[+\-*/]/, 'operator'],
                    
                    // Variables - identifiers (check against keyword lists)
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
        
        // ===== STEP 3: Define Token Colors =====
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
                { token: 'function.return', foreground: 'CE93D8', fontStyle: 'bold' },
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
        
        // ===== STEP 4: Add Autocomplete/Suggestions =====
        monaco.languages.registerCompletionItemProvider('pseudo-code', {
            provideCompletionItems: function(model, position) {
                var word = model.getWordUntilPosition(position);
                var range = {
                    startLineNumber: position.lineNumber,
                    endLineNumber: position.lineNumber,
                    startColumn: word.startColumn,
                    endColumn: word.endColumn
                };
                
                var suggestions = [];
                
                // Helper function to create suggestion
                function createSuggestion(label, kind, doc, insertText, snippet) {
                    var item = {
                        label: label,
                        kind: kind,
                        documentation: doc,
                        insertText: insertText,
                        range: range
                    };
                    if (snippet) {
                        item.insertTextRules = monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet;
                    }
                    return item;
                }
                
                // Packet construction commands
                suggestions.push(
                    createSuggestion('PACKET_START', monaco.languages.CompletionItemKind.Keyword, 'Marks the beginning of a packet definition', 'PACKET_START', false),
                    createSuggestion('PACKET_END', monaco.languages.CompletionItemKind.Keyword, 'Marks the end of a packet definition', 'PACKET_END', false),
                    createSuggestion('WRITE_BYTE', monaco.languages.CompletionItemKind.Function, 'Writes a single byte (0-255). Example: WRITE_BYTE 0xFF', 'WRITE_BYTE ${1:value}', true),
                    createSuggestion('WRITE_SHORT', monaco.languages.CompletionItemKind.Function, 'Writes a 16-bit integer (little-endian). Example: WRITE_SHORT 1234', 'WRITE_SHORT ${1:value}', true),
                    createSuggestion('WRITE_SHORT_BE', monaco.languages.CompletionItemKind.Function, 'Writes a 16-bit integer (big-endian/network byte order). Example: WRITE_SHORT_BE 1234', 'WRITE_SHORT_BE ${1:value}', true),
                    createSuggestion('WRITE_INT', monaco.languages.CompletionItemKind.Function, 'Writes a 32-bit integer (little-endian). Example: WRITE_INT 50000', 'WRITE_INT ${1:value}', true),
                    createSuggestion('WRITE_INT_BE', monaco.languages.CompletionItemKind.Function, 'Writes a 32-bit integer (big-endian/network byte order). Example: WRITE_INT_BE PACKET_LEN', 'WRITE_INT_BE ${1:value}', true),
                    createSuggestion('WRITE_VARINT', monaco.languages.CompletionItemKind.Function, 'Writes a variable-length integer (Minecraft-style). Example: WRITE_VARINT 300', 'WRITE_VARINT ${1:value}', true),
                    createSuggestion('WRITE_STRING', monaco.languages.CompletionItemKind.Function, 'Writes a null-terminated string. Example: WRITE_STRING "Hello Server"', 'WRITE_STRING "${1:text}"', true),
                    createSuggestion('WRITE_STRING_LEN', monaco.languages.CompletionItemKind.Function, 'Writes a fixed-length string. Example: WRITE_STRING_LEN "Test" 10', 'WRITE_STRING_LEN "${1:text}" ${2:length}', true),
                    createSuggestion('WRITE_BYTES', monaco.languages.CompletionItemKind.Function, 'Writes raw hexadecimal bytes. Example: WRITE_BYTES "FF00AA55"', 'WRITE_BYTES "${1:hex}"', true)
                );
                
                // Response parsing commands
                suggestions.push(
                    createSuggestion('RESPONSE_START', monaco.languages.CompletionItemKind.Keyword, 'Marks the beginning of response parsing rules', 'RESPONSE_START', false),
                    createSuggestion('RESPONSE_END', monaco.languages.CompletionItemKind.Keyword, 'Marks the end of response parsing rules', 'RESPONSE_END', false),
                    createSuggestion('READ_BYTE', monaco.languages.CompletionItemKind.Function, 'Reads a single byte and stores it in a variable. Example: READ_BYTE packet_id', 'READ_BYTE ${1:var_name}', true),
                    createSuggestion('READ_SHORT', monaco.languages.CompletionItemKind.Function, 'Reads a 16-bit integer (little-endian). Example: READ_SHORT player_count', 'READ_SHORT ${1:var_name}', true),
                    createSuggestion('READ_SHORT_BE', monaco.languages.CompletionItemKind.Function, 'Reads a 16-bit integer (big-endian). Example: READ_SHORT_BE port_number', 'READ_SHORT_BE ${1:var_name}', true),
                    createSuggestion('READ_INT', monaco.languages.CompletionItemKind.Function, 'Reads a 32-bit integer (little-endian). Example: READ_INT server_version', 'READ_INT ${1:var_name}', true),
                    createSuggestion('READ_INT_BE', monaco.languages.CompletionItemKind.Function, 'Reads a 32-bit integer (big-endian). Example: READ_INT_BE response_length', 'READ_INT_BE ${1:var_name}', true),
                    createSuggestion('READ_VARINT', monaco.languages.CompletionItemKind.Function, 'Reads a variable-length integer. Example: READ_VARINT packet_length', 'READ_VARINT ${1:var_name}', true),
                    createSuggestion('READ_STRING', monaco.languages.CompletionItemKind.Function, 'Reads a fixed-length string. Example: READ_STRING server_name 32', 'READ_STRING ${1:var_name} ${2:length}', true),
                    createSuggestion('READ_STRING_NULL', monaco.languages.CompletionItemKind.Function, 'Reads a null-terminated string. Example: READ_STRING_NULL server_name', 'READ_STRING_NULL ${1:var_name}', true),
                    createSuggestion('SKIP_BYTES', monaco.languages.CompletionItemKind.Function, 'Skips the specified number of bytes. Example: SKIP_BYTES 4', 'SKIP_BYTES ${1:count}', true)
                );
                
                // Validation commands
                suggestions.push(
                    createSuggestion('EXPECT_BYTE', monaco.languages.CompletionItemKind.Function, 'Validates that the next byte matches the expected value. Example: EXPECT_BYTE 0xFE', 'EXPECT_BYTE ${1:value}', true),
                    createSuggestion('EXPECT_MAGIC', monaco.languages.CompletionItemKind.Function, 'Validates that the next bytes match the expected magic bytes. Example: EXPECT_MAGIC "FEEDFACE"', 'EXPECT_MAGIC "${1:hex_string}"', true)
                );
                
                // Code block commands
                suggestions.push(
                    createSuggestion('CODE_START', monaco.languages.CompletionItemKind.Keyword, 'Marks the beginning of a code block', 'CODE_START', false),
                    createSuggestion('CODE_END', monaco.languages.CompletionItemKind.Keyword, 'Marks the end of a code block', 'CODE_END', false)
                );
                
                // Control flow
                suggestions.push(
                    createSuggestion('IF', monaco.languages.CompletionItemKind.Keyword, 'Conditional execution. Example: IF condition == 1:', 'IF ${1:condition}:', true),
                    createSuggestion('ELSE', monaco.languages.CompletionItemKind.Keyword, 'Else clause for IF statements', 'ELSE:', true),
                    createSuggestion('FOR', monaco.languages.CompletionItemKind.Keyword, 'Loop over an array. Example: FOR item IN array:', 'FOR ${1:var_name} IN ${2:array_name}:', true)
                );
                
                // Variable types
                suggestions.push(
                    createSuggestion('STRING', monaco.languages.CompletionItemKind.TypeParameter, 'Declare a string variable. Example: STRING name = "value"', 'STRING ${1:name} = ${2:"value"}', true),
                    createSuggestion('INT', monaco.languages.CompletionItemKind.TypeParameter, 'Declare an integer variable. Example: INT count = 10', 'INT ${1:name} = ${2:value}', true),
                    createSuggestion('BYTE', monaco.languages.CompletionItemKind.TypeParameter, 'Declare a byte variable. Example: BYTE status = 0xFF', 'BYTE ${1:name} = ${2:value}', true),
                    createSuggestion('FLOAT', monaco.languages.CompletionItemKind.TypeParameter, 'Declare a float variable. Example: FLOAT version = 1.19', 'FLOAT ${1:name} = ${2:value}', true),
                    createSuggestion('ARRAY', monaco.languages.CompletionItemKind.TypeParameter, 'Declare an array variable. Example: ARRAY items = ["a", "b", "c"]', 'ARRAY ${1:name} = ${2:["value1", "value2"]}', true)
                );
                
                // Functions
                suggestions.push(
                    createSuggestion('SPLIT', monaco.languages.CompletionItemKind.Function, 'Splits a string by delimiter. Example: SPLIT(var_name, ",")', 'SPLIT(${1:var_name}, ${2:"delimiter"})', true),
                    createSuggestion('REPLACE', monaco.languages.CompletionItemKind.Function, 'Replaces all occurrences in a string. Example: REPLACE(var_name, "old", "new")', 'REPLACE(${1:var_name}, ${2:"search"}, ${3:"replace"})', true),
                    createSuggestion('JSON_OUTPUT', monaco.languages.CompletionItemKind.Function, 'Parses a string variable as JSON. Example: JSON_OUTPUT JSON_PAYLOAD', 'JSON_OUTPUT ${1:var_name}', true),
                    createSuggestion('RETURN', monaco.languages.CompletionItemKind.Function, 'Formats the expression into Prometheus metric labels. Example: RETURN "server=HOST, protocol=1"', 'RETURN "${1:expression}"', true)
                );
                
                // Output blocks
                suggestions.push(
                    createSuggestion('OUTPUT_SUCCESS', monaco.languages.CompletionItemKind.Keyword, 'Marks output block that executes on success', 'OUTPUT_SUCCESS\n  ${1:// commands}\nOUTPUT_END', true),
                    createSuggestion('OUTPUT_ERROR', monaco.languages.CompletionItemKind.Keyword, 'Marks output block that executes on error', 'OUTPUT_ERROR\n  ${1:// commands}\nOUTPUT_END', true)
                );
                
                // Special placeholders
                suggestions.push(
                    createSuggestion('PACKET_LEN', monaco.languages.CompletionItemKind.Constant, 'Auto-calculated packet length placeholder', 'PACKET_LEN', false),
                    createSuggestion('HOST', monaco.languages.CompletionItemKind.Constant, 'Server hostname/address placeholder', 'HOST', false),
                    createSuggestion('PORT', monaco.languages.CompletionItemKind.Constant, 'Server port number placeholder', 'PORT', false),
                    createSuggestion('IP', monaco.languages.CompletionItemKind.Constant, 'Server IP address placeholder', 'IP', false),
                    createSuggestion('IP_LEN', monaco.languages.CompletionItemKind.Constant, 'Length of IP address string', 'IP_LEN', false),
                    createSuggestion('IP_LEN_HEX', monaco.languages.CompletionItemKind.Constant, 'Length of IP address in hexadecimal', 'IP_LEN_HEX', false)
                );
                
                return { suggestions: suggestions };
            }
        });
        
        // Mark language server as loaded
        if (typeof window !== 'undefined') {
            window.pseudoCodeLanguageServerLoaded = true;
        }
        
        // Trigger custom event
        if (typeof window !== 'undefined' && typeof CustomEvent !== 'undefined') {
            window.dispatchEvent(new CustomEvent('pseudoCodeLanguageServerReady'));
        }
    }
    
    // Start initialization when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initLanguageServer);
    } else {
        // DOM is already ready, start initialization
        initLanguageServer();
    }
})();

