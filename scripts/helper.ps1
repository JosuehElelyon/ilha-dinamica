# Processo auxiliar: volume, microfone e dispositivos de áudio (Core Audio),
# brilho (WMI) e teclas de mídia. Recebe um comando por linha no stdin e responde JSON no stdout.
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

Add-Type -TypeDefinition @'
using System;
using System.Text;
using System.Runtime.InteropServices;

[ComImport, Guid("5CDF2C82-841E-4546-9722-0CF74078229A"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IAudioEndpointVolume {
  int f(); int g(); int h(); int i();
  int SetMasterVolumeLevelScalar(float fLevel, ref Guid ctx);
  int j();
  int GetMasterVolumeLevelScalar(out float pfLevel);
  int k(); int l(); int m(); int n();
  int SetMute([MarshalAs(UnmanagedType.Bool)] bool bMute, ref Guid ctx);
  int GetMute([MarshalAs(UnmanagedType.Bool)] out bool pbMute);
}
[ComImport, Guid("0BD7A1BE-7A1A-44DB-8397-CC5392387B5E"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IMMDeviceCollection {
  int GetCount(out int count);
  int Item(int index, out IMMDevice device);
}
[ComImport, Guid("D666063F-1587-4E43-81F1-B948E807363F"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IMMDevice {
  int Activate(ref Guid id, int clsCtx, IntPtr activationParams, [MarshalAs(UnmanagedType.IUnknown)] out object iface);
  int OpenPropertyStore(int access, out IPropertyStore store);
  int GetId([MarshalAs(UnmanagedType.LPWStr)] out string id);
}
[ComImport, Guid("A95664D2-9614-4F35-A746-DE8DB63617E6"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IMMDeviceEnumerator {
  int EnumAudioEndpoints(int dataFlow, int stateMask, out IMMDeviceCollection devices);
  int GetDefaultAudioEndpoint(int dataFlow, int role, out IMMDevice endpoint);
}
[StructLayout(LayoutKind.Sequential)] struct PROPERTYKEY { public Guid fmtid; public int pid; }
[StructLayout(LayoutKind.Sequential)] struct PROPVARIANT { public short vt; short r1, r2, r3; public IntPtr p; IntPtr p2; }
[ComImport, Guid("886d8eeb-8cf2-4446-8d02-cdba1dbdcf99"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IPropertyStore {
  int GetCount(out int count);
  int GetAt(int index, out PROPERTYKEY key);
  int GetValue(ref PROPERTYKEY key, out PROPVARIANT value);
}
// Interface não documentada (usada por apps como EarTrumpet/AudioSwitcher) para trocar o dispositivo padrão
[ComImport, Guid("f8679f50-850a-41cf-9c72-430f290290c8"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IPolicyConfig {
  int a(); int b(); int c(); int d(); int e(); int f(); int g(); int h(); int i(); int j();
  int SetDefaultEndpoint([MarshalAs(UnmanagedType.LPWStr)] string id, int role);
}
[ComImport, Guid("BCDE0395-E52F-467C-8E3D-C4579291692E")] class MMDeviceEnumeratorComObject { }
[ComImport, Guid("870af99c-171d-4f9e-af0d-e63df40c2bc9")] class CPolicyConfigClient { }

public static class IlhaAudio {
  [DllImport("ole32.dll")] static extern int PropVariantClear(ref PROPVARIANT pvar);
  [DllImport("user32.dll")] static extern void keybd_event(byte b, byte s, uint f, UIntPtr e);

  static void Check(int hr) { Marshal.ThrowExceptionForHR(hr); }
  static IMMDeviceEnumerator En() { return (IMMDeviceEnumerator)(new MMDeviceEnumeratorComObject()); }
  static IAudioEndpointVolume VolOf(int flow) {
    IMMDevice d; Check(En().GetDefaultAudioEndpoint(flow, 0, out d));
    Guid id = typeof(IAudioEndpointVolume).GUID; object o;
    Check(d.Activate(ref id, 23, IntPtr.Zero, out o));
    return (IAudioEndpointVolume)o;
  }
  // flow: 0 = saída (fones/alto-falantes), 1 = entrada (microfone)
  public static int GetLevel(int flow) {
    try { float v; Check(VolOf(flow).GetMasterVolumeLevelScalar(out v)); return (int)Math.Round(v * 100); } catch { return -1; }
  }
  public static void SetLevel(int flow, int level) {
    Guid g = Guid.Empty; Check(VolOf(flow).SetMasterVolumeLevelScalar(Math.Max(0, Math.Min(100, level)) / 100f, ref g));
  }
  public static bool GetMute(int flow) {
    try { bool m; Check(VolOf(flow).GetMute(out m)); return m; } catch { return false; }
  }
  public static void SetMute(int flow, bool mute) { Guid g = Guid.Empty; Check(VolOf(flow).SetMute(mute, ref g)); }

  public static string VolJson() {
    return "{\"type\":\"vol\",\"level\":" + GetLevel(0) + ",\"mute\":" + (GetMute(0) ? "true" : "false") +
           ",\"mic\":" + GetLevel(1) + ",\"micMute\":" + (GetMute(1) ? "true" : "false") + "}";
  }

  static string Esc(string s) {
    var sb = new StringBuilder();
    foreach (char c in s ?? "") {
      if (c == '"' || c == '\\') sb.Append('\\').Append(c);
      else if (c < 32) sb.Append(' ');
      else sb.Append(c);
    }
    return sb.ToString();
  }
  static string Name(IMMDevice dev) {
    IPropertyStore ps;
    if (dev.OpenPropertyStore(0, out ps) != 0) return "Dispositivo";
    var key = new PROPERTYKEY { fmtid = new Guid("a45c254e-df1c-4efd-8020-67d146a850e0"), pid = 14 };
    PROPVARIANT v;
    if (ps.GetValue(ref key, out v) != 0) return "Dispositivo";
    string s = v.vt == 31 ? Marshal.PtrToStringUni(v.p) : "Dispositivo";
    PropVariantClear(ref v);
    return s;
  }
  static string List(int flow) {
    var en = En();
    string defId = "";
    try { IMMDevice d; if (en.GetDefaultAudioEndpoint(flow, 0, out d) == 0) d.GetId(out defId); } catch { }
    IMMDeviceCollection col; Check(en.EnumAudioEndpoints(flow, 1, out col));   // 1 = apenas ativos
    int n; col.GetCount(out n);
    var sb = new StringBuilder("[");
    for (int i = 0; i < n; i++) {
      IMMDevice dev; col.Item(i, out dev);
      string id; dev.GetId(out id);
      if (i > 0) sb.Append(',');
      sb.Append("{\"id\":\"" + Esc(id) + "\",\"name\":\"" + Esc(Name(dev)) + "\",\"def\":" + (id == defId ? "true" : "false") + "}");
    }
    return sb.Append(']').ToString();
  }
  public static string DevicesJson() {
    string o = "[]", i = "[]";
    try { o = List(0); } catch { }
    try { i = List(1); } catch { }
    return "{\"type\":\"devices\",\"out\":" + o + ",\"in\":" + i + "}";
  }
  public static void SetDefault(string id) {
    var pc = (IPolicyConfig)(new CPolicyConfigClient());
    for (int role = 0; role < 3; role++) Check(pc.SetDefaultEndpoint(id, role));
  }
  public static void Key(byte vk) { keybd_event(vk, 0, 0, UIntPtr.Zero); keybd_event(vk, 0, 2, UIntPtr.Zero); }
}
'@

function Say([string]$s) { [Console]::Out.WriteLine($s); [Console]::Out.Flush() }
function Send($o) { Say ($o | ConvertTo-Json -Compress) }
function Get-Bright {
  try { [int](Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightness -ErrorAction Stop | Select-Object -First 1).CurrentBrightness }
  catch { $null }
}

while ($true) {
  $line = [Console]::In.ReadLine()
  if ($line -eq $null) { break }
  $p = $line.Trim().Split(' ', 2)
  try {
    switch ($p[0]) {
      'vol'       { Say ([IlhaAudio]::VolJson()) }
      'setvol'    { [IlhaAudio]::SetLevel(0, [int]$p[1]); if ([IlhaAudio]::GetMute(0)) { [IlhaAudio]::SetMute(0, $false) }; Say ([IlhaAudio]::VolJson()) }
      'mute'      { [IlhaAudio]::SetMute(0, -not [IlhaAudio]::GetMute(0)); Say ([IlhaAudio]::VolJson()) }
      'setmic'    { [IlhaAudio]::SetLevel(1, [int]$p[1]); Say ([IlhaAudio]::VolJson()) }
      'micmute'   { [IlhaAudio]::SetMute(1, -not [IlhaAudio]::GetMute(1)); Say ([IlhaAudio]::VolJson()) }
      'devices'   { Say ([IlhaAudio]::DevicesJson()) }
      'setdev'    { [IlhaAudio]::SetDefault($p[1]); Start-Sleep -Milliseconds 200; Say ([IlhaAudio]::DevicesJson()); Say ([IlhaAudio]::VolJson()) }
      'bright'    { Send @{ type = 'bright'; level = (Get-Bright) } }
      'setbright' {
        $m = Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightnessMethods -ErrorAction Stop | Select-Object -First 1
        $null = $m.WmiSetBrightness(1, [byte][int]$p[1])
        Send @{ type = 'bright'; level = [int]$p[1] }
      }
      'key'       { [IlhaAudio]::Key([byte][Convert]::ToInt32($p[1], 16)) }
    }
  } catch {
    if ($p[0] -like '*bright*') { Send @{ type = 'bright'; level = $null } }
    else { Send @{ type = 'error'; cmd = $p[0]; msg = "$($_.Exception.Message)" } }
  }
}
