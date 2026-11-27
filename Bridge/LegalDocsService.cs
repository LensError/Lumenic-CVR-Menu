using System;
using System.Collections.Generic;
using System.IO;
using System.Net;
using System.Net.Http;
using System.Text;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using MelonLoader;

namespace LumenicMenu.Bridge;

internal sealed class LegalBlockDto
{
    public string kind = "p"; // "h1" | "h2" | "h3" | "p" | "li"
    public string text = string.Empty;
}

internal sealed class LegalDocDto
{
    public string id = string.Empty;
    public string title = string.Empty;
    public string url = string.Empty;
    public string status = "idle"; // "idle" | "loading" | "ready" | "error"
    public string error = string.Empty;
    public List<LegalBlockDto> blocks = new();
}

/// <summary>
/// Lazily fetches and parses ChilloutVR's official legal documentation from docs.chilloutvr.net
/// into clean structured text blocks for the in-game Settings -> Legal viewer.
/// </summary>
internal sealed class LegalDocsService
{
    private static readonly HttpClient Http = new()
    {
        Timeout = TimeSpan.FromSeconds(12),
    };

    private static readonly (string Id, string Title, string Url)[] DocDefs =
    {
        ("tos", "Terms of Service", "https://docs.chilloutvr.net/official/legal/tos/"),
        ("behaviour", "Behaviour Guidelines", "https://docs.chilloutvr.net/official/legal/behaviour_guidelines/"),
        ("content", "Content Guidelines", "https://docs.chilloutvr.net/official/legal/content_guidelines/"),
        ("privacy", "Privacy Policy", "https://docs.chilloutvr.net/official/legal/privacy-policy/"),
    };

    private const int MaxDocBytes = 2 * 1024 * 1024;
    private static readonly TimeSpan RegexTimeout = TimeSpan.FromSeconds(2);

    private static readonly Regex StripNoiseRegex = new(
        @"<(script|style|nav|svg|button)[^>]*>[\s\S]*?</\1>|<a[^>]*class=""[^""]*headerlink[^""]*""[^>]*>[\s\S]*?</a>",
        RegexOptions.Singleline | RegexOptions.IgnoreCase | RegexOptions.Compiled,
        RegexTimeout);

    private static readonly Regex BlockRegex = new(
        @"<(?<tag>h1|h2|h3|h4|p|li)\b[^>]*>(?<inner>[\s\S]*?)</\k<tag>>",
        RegexOptions.Singleline | RegexOptions.IgnoreCase | RegexOptions.Compiled,
        RegexTimeout);

    private static readonly Regex HtmlTagRegex = new(
        @"<[^>]+>",
        RegexOptions.Singleline | RegexOptions.Compiled,
        RegexTimeout);

    private static readonly Regex WhitespaceRegex = new(
        @"\s+",
        RegexOptions.Compiled,
        RegexTimeout);

    private readonly object _lock = new();
    private readonly Action _markDirty;
    private readonly Dictionary<string, LegalDocDto> _docs = new(StringComparer.Ordinal);

    public LegalDocsService(Action markDirty)
    {
        _markDirty = markDirty;
        foreach (var (id, title, url) in DocDefs)
        {
            _docs[id] = new LegalDocDto
            {
                id = id,
                title = title,
                url = url,
                status = "idle",
            };
        }
    }

    public void FetchDoc(string docId, bool force = false)
    {
        if (string.IsNullOrWhiteSpace(docId))
            return;

        string url;
        lock (_lock)
        {
            if (!_docs.TryGetValue(docId, out LegalDocDto doc))
                return;
            if (!force && (doc.status == "loading" || doc.status == "ready"))
                return;

            doc.status = "loading";
            doc.error = string.Empty;
            url = doc.url;
        }

        _markDirty?.Invoke();

        Task.Run(async () =>
        {
            try
            {
                using var req = new HttpRequestMessage(HttpMethod.Get, url);
                req.Headers.UserAgent.ParseAdd("LumenicMenu/" + LumenicMenuMod.ModVersion);
                using HttpResponseMessage resp = await Http.SendAsync(req, HttpCompletionOption.ResponseHeadersRead).ConfigureAwait(false);
                if (!resp.IsSuccessStatusCode)
                    throw new Exception($"HTTP {(int)resp.StatusCode} {resp.ReasonPhrase}");

                string html = await ReadBoundedStringAsync(resp.Content, MaxDocBytes).ConfigureAwait(false);
                List<LegalBlockDto> parsed = ParseArticleBlocks(html);
                if (parsed.Count == 0)
                    throw new Exception("No readable document content found.");

                lock (_lock)
                {
                    if (_docs.TryGetValue(docId, out LegalDocDto doc))
                    {
                        doc.blocks = parsed;
                        doc.status = "ready";
                        doc.error = string.Empty;
                    }
                }
            }
            catch (Exception ex)
            {
                MelonLogger.Warning($"[LumenicMenu] Failed to fetch legal doc '{docId}': {ex.Message}");
                lock (_lock)
                {
                    if (_docs.TryGetValue(docId, out LegalDocDto doc))
                    {
                        doc.status = "error";
                        doc.error = ex.Message;
                    }
                }
            }

            _markDirty?.Invoke();
        });
    }

    public List<LegalDocDto> BuildSnapshot()
    {
        var list = new List<LegalDocDto>(DocDefs.Length);
        lock (_lock)
        {
            foreach (var (id, _, _) in DocDefs)
            {
                if (_docs.TryGetValue(id, out LegalDocDto doc))
                {
                    list.Add(new LegalDocDto
                    {
                        id = doc.id,
                        title = doc.title,
                        url = doc.url,
                        status = doc.status,
                        error = doc.error,
                        blocks = doc.blocks != null ? new List<LegalBlockDto>(doc.blocks) : new List<LegalBlockDto>(),
                    });
                }
            }
        }
        return list;
    }

    private static List<LegalBlockDto> ParseArticleBlocks(string html)
    {
        var blocks = new List<LegalBlockDto>();
        if (string.IsNullOrEmpty(html))
            return blocks;

        string scope = html;
        int artStart = html.IndexOf("<article", StringComparison.OrdinalIgnoreCase);
        if (artStart >= 0)
        {
            int artClose = html.IndexOf(">", artStart, StringComparison.Ordinal);
            int artEnd = html.IndexOf("</article>", artStart, StringComparison.OrdinalIgnoreCase);
            if (artClose > artStart && artEnd > artClose)
                scope = html.Substring(artClose + 1, artEnd - artClose - 1);
        }

        scope = StripNoiseRegex.Replace(scope, string.Empty);

        foreach (Match m in BlockRegex.Matches(scope))
        {
            string tag = m.Groups["tag"].Value.ToLowerInvariant();
            string inner = m.Groups["inner"].Value;

            // Avoid duplicating nested <p> inside <li> by stripping inner <p> tags cleanly
            string text = WebUtility.HtmlDecode(HtmlTagRegex.Replace(inner, " "));
            text = WhitespaceRegex.Replace(text, " ").Trim();
            if (string.IsNullOrEmpty(text))
                continue;

            string kind = tag switch
            {
                "h1" => "h1",
                "h2" => "h2",
                "h3" or "h4" => "h3",
                "li" => "li",
                _ => "p",
            };

            // Skip exact duplicate consecutive line (e.g. <p> nested directly inside <li>)
            if (blocks.Count > 0 && blocks[blocks.Count - 1].text == text)
                continue;

            blocks.Add(new LegalBlockDto { kind = kind, text = text });
        }

        return blocks;
    }

    private static async Task<string> ReadBoundedStringAsync(HttpContent content, int maxBytes)
    {
        long? contentLen = content.Headers.ContentLength;
        if (contentLen.HasValue && contentLen.Value > maxBytes)
            throw new Exception("Legal document response exceeded maximum size.");

        using Stream stream = await content.ReadAsStreamAsync().ConfigureAwait(false);
        using var ms = new MemoryStream();
        byte[] buffer = new byte[16384];
        int total = 0;
        while (true)
        {
            int read = await stream.ReadAsync(buffer, 0, buffer.Length).ConfigureAwait(false);
            if (read <= 0)
                break;
            total += read;
            if (total > maxBytes)
                throw new Exception("Legal document response exceeded maximum size.");
            ms.Write(buffer, 0, read);
        }
        return Encoding.UTF8.GetString(ms.GetBuffer(), 0, (int)ms.Length);
    }
}
