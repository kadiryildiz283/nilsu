local M = {}

local socket_path = "/tmp/nilsu.sock"

function M.get_context()
  local uv = vim.loop or vim.uv
  local client = uv.new_pipe(false)
  
  local file = vim.api.nvim_buf_get_name(0)
  local cursor_line = vim.api.nvim_win_get_cursor(0)[1]

  if file == "" then
    print("[Nilsu] Buffer has no file name")
    return
  end

  local request = vim.json.encode({
    action = "get_context",
    file = file,
    cursor_line = cursor_line
  })

  client:connect(socket_path, function(err)
    if err then
      vim.schedule(function()
        print("[Nilsu] Connection error: " .. tostring(err))
      end)
      return
    end

    client:write(request, function(write_err)
      if write_err then
        vim.schedule(function()
          print("[Nilsu] Write error: " .. tostring(write_err))
        end)
        client:close()
        return
      end
    end)

    local response_chunks = {}
    client:read_start(function(read_err, chunk)
      if read_err then
        vim.schedule(function()
          print("[Nilsu] Read error: " .. tostring(read_err))
        end)
        client:close()
        return
      end

      if chunk then
        table.insert(response_chunks, chunk)
      else
        -- EOF
        client:close()
        local full_response = table.concat(response_chunks)
        vim.schedule(function()
          local ok, decoded = pcall(vim.json.decode, full_response)
          if not ok then
            print("[Nilsu] Failed to decode response: " .. full_response)
            return
          end

          if decoded.status == "ok" and decoded.context_snippet then
            local snippet = decoded.context_snippet
            M.show_floating_window(snippet.code_snippet, snippet.start_line, snippet.end_line)
          else
            print("[Nilsu] No context found or error: " .. (decoded.message or "unknown"))
          end
        end)
      end
    end)
  end)
end

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
