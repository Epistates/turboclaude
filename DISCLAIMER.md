# Disclaimer

## TurboClaude is an Unofficial Community-Maintained Project

**TurboClaude is NOT created, maintained, endorsed, or officially supported by Anthropic.** This is a community-maintained Rust SDK for the Claude API.

## Official Anthropic SDKs

For official SDKs and support, please visit:
- **Official Anthropic SDKs**: https://github.com/anthropics
- **Official Documentation**: https://docs.anthropic.com/

## Attribution and References

This SDK was developed as an **unofficial Rust port** using the following official Anthropic SDKs as references for feature parity:

### Reference Implementations
- **Python SDK** (`anthropic-sdk-python`) - Used as primary reference for API coverage and feature design
- **Claude Agent SDK** (`claude-agent-sdk-python`) - Reference for agent framework and tool execution patterns
- **Skills** (`skills`) - Reference for skill registration, discovery, and execution systems
- **Official Anthropic Documentation** - For complete feature set and API specifications

All reference SDKs are maintained by Anthropic and released under the MIT License.

### Feature Parity Approach

The TurboClaude SDK aims for 100% feature parity with the official Python SDK, including:
- Messages API (with streaming)
- Batch processing
- Tool use and function calling
- Beta features
- Document and PDF analysis
- Prompt caching
- Token counting
- Error handling and retry logic
- Rate limiting
- Model information endpoints

This is achieved through careful study of the official SDKs' implementation, behavior, and API design patterns—**NOT through any unauthorized use of Anthropic's proprietary code**.

## Licensing

- **TurboClaude**: MIT License (see [LICENSE](LICENSE))
- **Reference SDKs**: MIT License (see https://github.com/anthropics)

Both this project and the reference implementations are under MIT, allowing free use, modification, and distribution with proper attribution.

## Important Notes

1. **Not Endorsed by Anthropic**: While we respect and reference Anthropic's excellent SDK implementations, TurboClaude is not endorsed, maintained, or supported by Anthropic.

2. **Community Support**: Issues, bug reports, and contributions should be directed to the TurboClaude repository, not to Anthropic.

3. **API Key Security**: Like all official SDKs, TurboClaude never stores or logs your API keys. Always keep your `ANTHROPIC_API_KEY` private and secure.

4. **API Stability**: This SDK follows the Claude API as documented at https://docs.anthropic.com/. API changes may require SDK updates.

5. **Production Use**: While TurboClaude is production-ready and thoroughly tested, you use it at your own discretion. Ensure it meets your security and reliability requirements before using in production environments.

## Credits

- **Maintained by**: [Epistates](https://github.com/epistates)
- **Reference SDKs**: Anthropic's official Python, TypeScript, and documentation teams
- **Community Contributors**: All who have reported issues or contributed improvements

## Support

For issues with TurboClaude:
- GitHub Issues: Report bugs and request features on the TurboClaude repository
- Discussions: Use the repository discussions for questions and general support

For issues with the Claude API itself:
- Official Support: Contact Anthropic through their official support channels
- Documentation: Consult https://docs.anthropic.com/

## NO Relationship to Anthropic

This project is **not affiliated with, endorsed by, or officially supported by Anthropic**. We are an independent open-source project that provides a Rust implementation of the Claude API following Anthropic's official specifications and best practices.

We respect Anthropic's work and recommend using the official SDKs when available in your language. TurboClaude exists to serve the Rust community with a native, idiomatic implementation.

---

**Last Updated**: 2025

For the complete legal terms, see the [LICENSE](LICENSE) file.
