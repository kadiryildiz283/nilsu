local M = {}

local socket_path = "/tmp/nilsu.sock"

--- Low-level query function to fetch AST context asynchronously from the daemon.
-- @param file string The absolute file path.
-- @param cursor_line number The cursor line (1-indexed).
-- @param callback fun(err: string|nil, decoded: table|nil) Callback invoked on completion.
function M.query_context(file, cursor_line, callback)
  local uv = vim.loop or vim.uv
  local client = uv.new_pipe(false)

  local request = vim.json.encode({
    action = "get_context",
    file = file,
    cursor_line = cursor_line
  })

  client:connect(socket_path, function(err)
    if err then
      callback(tostring(err), nil)
      return
    end

    client:write(request, function(write_err)
      if write_err then
        client:close()
        callback(tostring(write_err), nil)
        return
      end
    end)

    local response_chunks = {}
    client:read_start(function(read_err, chunk)
      if read_err then
        client:close()
        callback(tostring(read_err), nil)
        return
      end

      if chunk then
        table.insert(response_chunks, chunk)
      else
        client:close()
        local full_response = table.concat(response_chunks)
        local ok, decoded = pcall(vim.json.decode, full_response)
        if not ok then
          callback("Failed to decode JSON response: " .. full_response, nil)
          return
        end
        callback(nil, decoded)
      end
    end)
  end)
end

--- Fetches context and shows it in a floating window.
function M.get_context()
  local file = vim.api.nvim_buf_get_name(0)
  local cursor_line = vim.api.nvim_win_get_cursor(0)[1]

  if file == "" then
    print("[Nilsu] Buffer has no file name")
    return
  end

  M.query_context(file, cursor_line, function(err, decoded)
    vim.schedule(function()
      if err then
        print("[Nilsu] Connection or query error: " .. err)
        return
      end

      if decoded.status == "ok" then
        local snippet = decoded.context_snippet
        if snippet == nil or snippet == vim.NIL then
          print("[Nilsu] No meaningful AST context found at the cursor position.")
          return
        end
        M.show_floating_window(snippet.code_snippet, snippet.start_line, snippet.end_line)
      else
        print("[Nilsu] Error: " .. (decoded.message or "unknown"))
      end
    end)
  end)
end

--- Utility function to open a clean floating window.
function M.show_floating_window(content, start_line, end_line)
  local lines = vim.split(content, "\n")
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  
  -- Set options for display
  vim.api.nvim_buf_set_option(buf, "filetype", "rust")
  vim.api.nvim_buf_set_option(buf, "bufhidden", "wipe")

  -- Calculate size and position
  local width = math.min(80, vim.o.columns - 4)
  local height = math.min(#lines, vim.o.lines - 4)
  local row = math.floor((vim.o.lines - height) / 2)
  local col = math.floor((vim.o.columns - width) / 2)

  local opts = {
    relative = "editor",
    width = width,
    height = height,
    row = row,
    col = col,
    style = "minimal",
    border = "rounded",
    title = string.format(" Nilsu Context (Lines %d-%d) ", start_line, end_line),
    title_pos = "center",
  }

  local win = vim.api.nvim_open_win(buf, true, opts)
  
  -- Map 'q' to close the window
  vim.api.nvim_buf_set_keymap(buf, 'n', 'q', ':q<CR>', { noremap = true, silent = true })
end

return M
