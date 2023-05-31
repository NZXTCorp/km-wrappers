#define _AMD64_

// no idea why this is needed to compile - AFAICT this is define is set exactly this way in the
// headers already
#define DECLSPEC_DEPRECATED_DDK DECLSPEC_DEPRECATED

// Allow Windows 10 DDK API
#define NTDDI_VERSION NTDDI_WIN10
#define _WIN32_WINNT _WIN32_WINNT_WIN10
#define WINVER _WIN32_WINNT_WIN10

#include <sdkddkver.h>

// needed for security descriptor
#include <ntifs.h>

// basic NT DDK types
#include <ntddk.h>

// Windows Driver Model
#include <wdm.h>
// Windows Driver Framework
#include <wdf.h>
#include <wdfdriver.h>
