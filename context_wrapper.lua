local M = {}

--- Formats the context snippet from Nilsu into a clean prompt block template.
-- @param context_snippet table The context_snippet object from Nilsu containing:
--   - code_snippet: string (the actual code block)
--   - start_line: number (1-indexed start line)
--   - end_line: number (1-indexed end line)
-- @param filepath string The path of the source file.
-- @return string A formatted context envelope prompt string.
function M.wrap_context(context_snippet, filepath)
  if not context_snippet or context_snippet == vim.NIL then
    return ""
  end

  local code = context_snippet.code_snippet
  local start_line = context_snippet.start_line
  local end_line = context_snippet.end_line

  -- Standardized ground-truth context prompt envelope
  local template = string.format([=[
--- CONTEXT START ---
File: %s
Lines: %d-%d
Code:
```rust
%s
```
--- CONTEXT END ---
]=], filepath, start_line, end_line, code)

  return template
end

--- Programmatically retrieves context from Nilsu and returns the wrapped prompt string.
-- This function runs asynchronously.
-- @param callback fun(err: string|nil, prompt: string|nil) Callback function.
function M.get_wrapped_context(callback)
  local nilsu = require("nilsu")
  local file = vim.api.nvim_buf_get_name(0)
  local cursor_line = vim.api.nvim_win_get_cursor(0)[1]

  if file == "" then
    callback("Buffer has no file name", nil)
    return
  end

  nilsu.query_context(file, cursor_line, function(err, decoded)
    if err then
      callback(err, nil)
      return
    end

    if decoded.status == "ok" then
      local snippet = decoded.context_snippet
      if snippet == nil or snippet == vim.NIL then
        callback("No meaningful AST context found", nil)
        return
      end
      local wrapped = M.wrap_context(snippet, file)
      callback(nil, wrapped)
    else
      callback(decoded.message or "unknown error", nil)
    end
  end)
end

return M
