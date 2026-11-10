# Installs or updates Youth Center Sign In on the office computer.
#
# Run it by double-clicking "Install.cmd" in the same folder. It is safe to
# run again: an update keeps your settings and all sign-in data.
#
# What it does:
#   1. Copies the program to C:\YouthCenterSignIn
#   2. Makes it start by itself when this user signs in to Windows
#   3. Lets the tablet reach it through Windows Firewall (asks for permission)
#   4. Stops this computer from going to sleep while plugged in
#   5. Starts the program and opens the staff page

param(
    # Used when the script restarts itself as administrator for step 3 and 4.
    [switch]$AdminPart,
    [string]$ExePath,
    [switch]$MakeNetworkPrivate
)

$ErrorActionPreference = 'Stop'
$AppName = 'Youth Center Sign In'
$InstallFolder = 'C:\YouthCenterSignIn'

# ---------------------------------------------------------------------------
# The part that needs administrator rights. It runs in its own window.
# ---------------------------------------------------------------------------
if ($AdminPart) {
    try {
        # Replace our firewall rule, so updates don't pile up duplicates.
        Get-NetFirewallRule -DisplayName $AppName -ErrorAction SilentlyContinue | Remove-NetFirewallRule
        New-NetFirewallRule -DisplayName $AppName -Direction Inbound -Program $ExePath `
            -Action Allow -Profile Private, Domain | Out-Null

        # Windows only lets other devices in on networks marked "Private".
        if ($MakeNetworkPrivate) {
            Get-NetConnectionProfile | Where-Object { $_.NetworkCategory -eq 'Public' } |
                Set-NetConnectionProfile -NetworkCategory Private
        }

        # Never sleep or hibernate while plugged in.
        powercfg /change standby-timeout-ac 0
        powercfg /change hibernate-timeout-ac 0
        exit 0
    }
    catch {
        Write-Host "Something went wrong: $_" -ForegroundColor Red
        Start-Sleep -Seconds 10   # time to read the message before the window closes
        exit 1
    }
}

# ---------------------------------------------------------------------------
# The normal part, run as the person who uses this computer.
# ---------------------------------------------------------------------------
function Say($text) { Write-Host $text }
function Step($text) { Write-Host ''; Write-Host $text -ForegroundColor Cyan }

$Source = $PSScriptRoot
$Exe = Join-Path $InstallFolder 'signin.exe'

if (-not (Test-Path (Join-Path $Source 'signin.exe'))) {
    Say 'signin.exe is not in this folder.'
    Say 'Unzip the whole download first, then run Install.cmd from the unzipped folder.'
    exit 1
}

Step "1. Copying the program to $InstallFolder"
$updating = Test-Path $Exe
Get-Process -Name signin -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 500
New-Item -ItemType Directory -Force -Path $InstallFolder | Out-Null
Copy-Item (Join-Path $Source 'signin.exe') $Exe -Force
Unblock-File $Exe
if (Test-Path (Join-Path $Source 'README.md')) {
    Copy-Item (Join-Path $Source 'README.md') $InstallFolder -Force
}
# Settings are only copied the first time, so an update never overwrites them.
$Config = Join-Path $InstallFolder 'config.toml'
if (-not (Test-Path $Config)) {
    Copy-Item (Join-Path $Source 'config.toml') $Config
}
if ($updating) { Say '   Updated. Your settings and sign-in data were kept.' }

Step '2. Making it start when you sign in to Windows'
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut((Join-Path ([Environment]::GetFolderPath('Startup')) "$AppName.lnk"))
$shortcut.TargetPath = $Exe
$shortcut.WorkingDirectory = $InstallFolder
$shortcut.WindowStyle = 7   # start minimized
$shortcut.Save()

# The port is 8080 unless config.toml says otherwise.
$Port = 8080
$portLine = Select-String -Path $Config -Pattern '^\s*port\s*=\s*(\d+)' | Select-Object -First 1
if ($portLine) { $Port = [int]$portLine.Matches[0].Groups[1].Value }
$StaffPage = "http://localhost:$Port/admin"

# A shortcut to the staff page on the desktop.
$link = Join-Path ([Environment]::GetFolderPath('Desktop')) 'Sign In - Staff page.url'
Set-Content -Path $link -Value "[InternetShortcut]`r`nURL=$StaffPage" -Encoding ASCII

Step '3 and 4. Firewall and sleep settings'
$adminArgs = "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`" -AdminPart -ExePath `"$Exe`""
$public = Get-NetConnectionProfile -ErrorAction SilentlyContinue | Where-Object { $_.NetworkCategory -eq 'Public' }
if ($public) {
    Say "   Windows has this network ($($public.Name -join ', ')) marked as Public,"
    Say '   which blocks the tablet. It should be Private if this is the youth center''s own network.'
    $answer = ''
    try { $answer = Read-Host '   Mark it as Private? (Y/N)' } catch { }   # nobody to ask: leave it
    if ($answer -match '^[Yy]') { $adminArgs += ' -MakeNetworkPrivate' }
}
Say '   Windows will now ask "Do you want to allow this app to make changes?". Choose Yes.'
$adminWorked = $false
try {
    $process = Start-Process powershell -Verb RunAs -ArgumentList $adminArgs -Wait -PassThru
    $adminWorked = ($process.ExitCode -eq 0)
}
catch {
    # The person chose No, or this account can't approve it.
}
if (-not $adminWorked) {
    Write-Host '   Skipped. The tablet may not be able to connect until this is done.' -ForegroundColor Yellow
    Say '   Run Install.cmd again and choose Yes, or ask someone with an administrator password.'
}

Step '5. Starting the program'
Start-Process $Exe -WorkingDirectory $InstallFolder -WindowStyle Minimized
$running = $false
foreach ($try in 1..30) {
    try {
        Invoke-WebRequest -Uri "http://localhost:$Port/api/kiosk/config" -UseBasicParsing -TimeoutSec 2 | Out-Null
        $running = $true
        break
    }
    catch { Start-Sleep -Milliseconds 500 }
}
if (-not $running) {
    Write-Host '   The program did not start.' -ForegroundColor Red
    Say "   Look at the newest file in $InstallFolder\data\logs for the reason."
    exit 1
}
Start-Process $StaffPage

# The addresses the tablet can use.
$addresses = Get-NetIPAddress -AddressFamily IPv4 -ErrorAction SilentlyContinue |
    Where-Object { $_.IPAddress -notlike '127.*' -and $_.IPAddress -notlike '169.254.*' } |
    ForEach-Object { "http://$($_.IPAddress):$Port" }

Write-Host ''
Write-Host 'Done.' -ForegroundColor Green
Say "The staff page is open in your browser. There is also a shortcut on the desktop."
Say ''
Say 'On the tablet, open this address:'
Say "    http://$($env:COMPUTERNAME):$Port"
if ($addresses) {
    Say 'If that does not open, use this one instead:'
    foreach ($address in $addresses) { Say "    $address" }
}
Say ''
Say 'Next steps are in the README: set the Admin PIN, connect Outlook, set up the tablet.'
