# Net Sentinel Documentation

![Main Page](../images/main-page.png)

Welcome to the Net Sentinel documentation! This documentation explains how to write pseudo-code scripts for monitoring game servers.

## 📚 Documentation Structure

### For Beginners

1. **[Overview](00-overview.md)** - Start here! Learn what Net Sentinel is and what it does.
2. **[Beginner's Guide](01-beginners-guide.md)** - Step-by-step tutorial on writing your first pseudo-code script.
3. **[Examples](03-examples.md)** - Real-world examples for different game server protocols.
4. **[Hands-On Demo](05-hands-on-demo.md)** - Walk through complete RCON and Minecraft examples step-by-step.

### Reference

5. **[Syntax Reference](02-pseudo-code-syntax.md)** - Complete command reference. Use this as a quick lookup.
6. **[How It Works](04-how-it-works.md)** - Deep dive into the internal implementation. Understand how scripts are processed.

## 🚀 Quick Start

**New to pseudo-code?** Follow this path:
1. Read [Overview](00-overview.md) to understand what Net Sentinel does
2. Follow [Beginner's Guide](01-beginners-guide.md) to write your first script
3. Walk through [Hands-On Demo](05-hands-on-demo.md) for complete examples
4. Check [Examples](03-examples.md) for real-world patterns
5. Use [Syntax Reference](02-pseudo-code-syntax.md) as needed

**Already familiar?** Jump to:
- [Syntax Reference](02-pseudo-code-syntax.md) for command details
- [Examples](03-examples.md) for protocol-specific examples

## 📖 What is Pseudo-Code?

Pseudo-code is a simple scripting language that lets you describe:
- **What packets to send** to a game server
- **How to parse the response** from the server
- **How to format the output** for monitoring

Instead of writing complex code, you write simple, readable instructions:

```pseudo
PACKET_START
WRITE_BYTE 0xFF
WRITE_STRING "status"
PACKET_END

RESPONSE_START
READ_STRING_NULL server_info
RESPONSE_END
```

## 🎯 Common Use Cases

### Monitor Game Server Status
Check if a game server is online and get player count, version, etc.

### Custom Protocol Support
Add support for any game server protocol without modifying code.

### Prometheus Metrics
Export server status as Prometheus metrics for monitoring dashboards.

## 📝 Documentation Files

| File | Description | Audience |
|------|-------------|----------|
| [00-overview.md](00-overview.md) | What Net Sentinel is and how it works | Everyone |
| [01-beginners-guide.md](01-beginners-guide.md) | Step-by-step tutorial | Beginners |
| [02-pseudo-code-syntax.md](02-pseudo-code-syntax.md) | Complete command reference | All users |
| [03-examples.md](03-examples.md) | Real-world examples | All users |
| [04-how-it-works.md](04-how-it-works.md) | Internal implementation details | Advanced users |
| [05-hands-on-demo.md](05-hands-on-demo.md) | Complete walkthrough of RCON and Minecraft examples | All users |

## 🔍 Finding What You Need

### "How do I write a script for [protocol]?"
→ Check [Hands-On Demo](05-hands-on-demo.md) for complete walkthroughs, or [Examples](03-examples.md) for protocol-specific examples

### "What command do I use for [operation]?"
→ Check [Syntax Reference](02-pseudo-code-syntax.md) for command details

### "How does [feature] work?"
→ Check [How It Works](04-how-it-works.md) for implementation details

### "I'm completely new, where do I start?"
→ Start with [Overview](00-overview.md), then [Beginner's Guide](01-beginners-guide.md)

## 💡 Key Concepts

### Packets
A packet is a sequence of bytes sent to a server. You construct packets using `WRITE_*` commands.

### Responses
Servers respond with bytes. You parse responses using `READ_*` commands.

### Variables
Values extracted from responses are stored as variables. You can use variables in output formatting.

### Output Formatting
Format results for Prometheus metrics using `OUTPUT_SUCCESS` and `OUTPUT_ERROR` blocks.

## 🛠️ Example Workflow

1. **Understand the protocol** - Read the game server's protocol documentation
2. **Write packet construction** - Use `WRITE_*` commands to build packets
3. **Write response parsing** - Use `READ_*` commands to extract data
4. **Format output** - Use `OUTPUT_*` blocks to format results
5. **Test** - Test with real server and verify results

## 📚 Additional Resources

- Main project README: See the root `README.md` for project setup
- Existing documentation: Check `pseudo-code-docs.md` in the root directory
- Protocol examples: See `minecraft_pseudo-code.md` and `rcon.md` in the root directory

## 🤝 Contributing

Found an error or want to improve the documentation? Contributions are welcome!

## 📞 Getting Help

If you're stuck:
1. Walk through [Hands-On Demo](05-hands-on-demo.md) to see complete examples
2. Check the [Examples](03-examples.md) for similar use cases
3. Review the [Syntax Reference](02-pseudo-code-syntax.md) for command details
4. Read [How It Works](04-how-it-works.md) to understand the system better

---

**Happy scripting!** 🎮

