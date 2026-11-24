using System;
using System.Collections.Generic;

namespace LumenicMenu.Bridge;

internal sealed class SystemAlertDto
{
    public long id;
    public string kind = "alert";
    public string title = string.Empty;
    public string message = string.Empty;
    public string confirm_id = string.Empty;
}

internal sealed class SystemAlertService
{
    private readonly object _lock = new();
    private readonly Queue<SystemAlertDto> _queue = new();
    private readonly Action _markDirty;
    private long _nextId = 1;

    public SystemAlertService(Action markDirty)
    {
        _markDirty = markDirty;
    }

    public void EnqueueAlert(string title, string message)
    {
        if (string.IsNullOrWhiteSpace(title) && string.IsNullOrWhiteSpace(message))
            return;

        lock (_lock)
        {
            _queue.Enqueue(new SystemAlertDto
            {
                id = _nextId++,
                kind = "alert",
                title = string.IsNullOrWhiteSpace(title) ? "System Notice" : title.Trim(),
                message = (message ?? string.Empty).Trim(),
                confirm_id = string.Empty,
            });
        }
        _markDirty?.Invoke();
    }

    public void EnqueueConfirm(string title, string message, string confirmId)
    {
        if (string.IsNullOrWhiteSpace(title) && string.IsNullOrWhiteSpace(message))
            return;

        lock (_lock)
        {
            _queue.Enqueue(new SystemAlertDto
            {
                id = _nextId++,
                kind = "confirm",
                title = string.IsNullOrWhiteSpace(title) ? "System Confirmation" : title.Trim(),
                message = (message ?? string.Empty).Trim(),
                confirm_id = confirmId ?? string.Empty,
            });
        }
        _markDirty?.Invoke();
    }

    public SystemAlertDto PeekActive()
    {
        lock (_lock)
        {
            return _queue.Count > 0 ? _queue.Peek() : null;
        }
    }

    public SystemAlertDto Dismiss(long id)
    {
        SystemAlertDto removed = null;
        lock (_lock)
        {
            if (_queue.Count > 0 && (_queue.Peek().id == id || id <= 0))
                removed = _queue.Dequeue();
        }
        if (removed != null)
            _markDirty?.Invoke();
        return removed;
    }
}
