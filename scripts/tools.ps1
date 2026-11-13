# Processo auxiliar para tarefas pesadas: lista de processos, lixeira e otimização de memória.
# Recebe um comando por linha no stdin e responde JSON no stdout.
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class IlhaMem {
  [DllImport("psapi.dll")] static extern bool EmptyWorkingSet(IntPtr h);
  // Pede ao Windows para devolver a memória ociosa de cada processo (o que der acesso)
  public static int TrimAll() {
    int n = 0;
    foreach (var p in System.Diagnostics.Process.GetProcesses()) {
      try { if (EmptyWorkingSet(p.Handle)) n++; } catch { } finally { p.Dispose(); }
    }
    return n;
  }
}
'@

function Say($o) { [Console]::Out.WriteLine(($o | ConvertTo-Json -Compress -Depth 4)); [Console]::Out.Flush() }
$cores = [Environment]::ProcessorCount

function RecycleInfo {
  $sh = New-Object -ComObject Shell.Application
  $items = @($sh.NameSpace(10).Items())
  $size = [int64]0
  foreach ($i in $items) { try { $size += [int64]$i.Size } catch { } }
  @{ type = 'recycle'; size = $size; count = $items.Count }
}

while ($true) {
  $line = [Console]::In.ReadLine()
  if ($line -eq $null) { break }
  $cmd = $line.Trim()
  switch ($cmd) {
    'procs' {
      try {
        $list = foreach ($x in (Get-CimInstance Win32_PerfFormattedData_PerfProc_Process -Property Name, IDProcess, PercentProcessorTime, WorkingSetPrivate -ErrorAction Stop)) {
          if ($x.IDProcess -eq 0 -or $x.Name -eq '_Total') { continue }
          [ordered]@{ n = ($x.Name -replace '#\d+$', ''); p = [int]$x.IDProcess; c = [math]::Round($x.PercentProcessorTime / $cores, 1); m = [int64]$x.WorkingSetPrivate }
        }
        Say @{ type = 'procs'; list = @($list) }
      } catch { Say @{ type = 'procs'; list = @(); error = "$($_.Exception.Message)" } }
    }
    'recycle' {
      try { Say (RecycleInfo) } catch { Say @{ type = 'recycle'; size = 0; count = 0; error = "$($_.Exception.Message)" } }
    }
    'emptyrecycle' {
      try { Clear-RecycleBin -Force -ErrorAction SilentlyContinue; Say (RecycleInfo) }
      catch { Say @{ type = 'recycle'; size = 0; count = 0; error = "$($_.Exception.Message)" } }
    }
    'trimram' {
      try { Say @{ type = 'trimram'; n = [IlhaMem]::TrimAll() } } catch { Say @{ type = 'trimram'; n = 0 } }
    }
  }
}
