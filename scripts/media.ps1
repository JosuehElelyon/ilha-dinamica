# Lê a mídia que está tocando no Windows (Spotify, YouTube no navegador, etc.)
# usando a API de controle de mídia do sistema e imprime uma linha JSON quando muda.
# Inclui a capa do álbum em base64 (data URL).
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
Add-Type -AssemblyName System.Runtime.WindowsRuntime

$asTaskGeneric = ([System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object {
  $_.Name -eq 'AsTask' -and $_.GetParameters().Count -eq 1 -and
  $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1'
})[0]

function Await($op, [Type]$t) {
  $task = $asTaskGeneric.MakeGenericMethod($t).Invoke($null, @($op))
  $task.Wait(-1) | Out-Null
  $task.Result
}

$null = [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager, Windows.Media.Control, ContentType = WindowsRuntime]
$null = [Windows.Storage.Streams.DataReader, Windows.Storage.Streams, ContentType = WindowsRuntime]
$mgrType    = [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager]
$propType   = [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionMediaProperties]
$streamType = [Windows.Storage.Streams.IRandomAccessStreamWithContentType]
$mgr = Await ($mgrType::RequestAsync()) $mgrType

function Get-Cover($props) {
  try {
    if (-not $props.Thumbnail) { return '' }
    $st = Await ($props.Thumbnail.OpenReadAsync()) $streamType
    $size = [uint32]$st.Size
    if ($size -eq 0) { return '' }
    $rd = [Windows.Storage.Streams.DataReader]::new($st.GetInputStreamAt(0))
    $null = Await ($rd.LoadAsync($size)) ([uint32])
    $bytes = New-Object byte[] $size
    $rd.ReadBytes($bytes)
    $ct = $st.ContentType
    if (-not $ct) { $ct = 'image/png' }
    $rd.Dispose(); $st.Dispose()
    return "data:$ct;base64," + [Convert]::ToBase64String($bytes)
  } catch { return '' }
}

$last = ''; $lastKey = ''; $cover = ''; $tries = 0
while ($true) {
  try {
    $s = $mgr.GetCurrentSession()
    if ($s) {
      $p = Await ($s.TryGetMediaPropertiesAsync()) $propType
      $i = $s.GetPlaybackInfo()
      $key = "$($p.Title)|$($p.Artist)"
      if ($key -ne $lastKey) { $lastKey = $key; $cover = ''; $tries = 0 }
      # a capa às vezes chega um pouco depois da troca de faixa: tenta algumas vezes
      if (-not $cover -and $tries -lt 4) { $tries++; $cover = Get-Cover $p }
      $o = [ordered]@{ title = "$($p.Title)"; artist = "$($p.Artist)"; status = $i.PlaybackStatus.ToString(); app = "$($s.SourceAppUserModelId)"; cover = $cover }
    } else {
      $o = [ordered]@{ title = ''; artist = ''; status = 'None'; app = ''; cover = '' }
    }
    $j = $o | ConvertTo-Json -Compress
    if ($j -ne $last) { [Console]::Out.WriteLine($j); [Console]::Out.Flush(); $last = $j }
  } catch { }
  Start-Sleep -Milliseconds 1000
}
