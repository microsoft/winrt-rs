use bindings::windows::win32;
use bindings::windows::win32::system_services::{BOOL, PWSTR};
use win32::intl;

fn main() -> windows::Result<()> {
    let input = std::env::args()
        .nth(1)
        .expect("Expected one command line argument for text to be spell-corrected");
    // Initialize the COM runtime for this thread
    windows::initialize_mta()?;

    // Create ISpellCheckerFactory
    let factory: intl::ISpellCheckerFactory = windows::create_instance(&intl::SpellCheckerFactory)?;

    // Make sure that the "en-US" locale is supported
    let mut supported: BOOL = false.into();
    let locale = "en-US";
    unsafe { factory.IsSupported(locale, &mut supported).ok()? };
    supported.expect("en-US is supported");

    // Create a ISpellChecker
    let mut checker = None;
    unsafe { factory.CreateSpellChecker(locale, &mut checker).ok()? };
    let checker = checker.unwrap();

    // Get errors enumerator for the supplied string
    let mut errors = None;
    unsafe {
        println!("Checking the text: '{}'", input);
        checker
            .ComprehensiveCheck(input.clone(), &mut errors)
            .ok()?
    };
    let errors = errors.unwrap();

    // Loop through all the errors
    loop {
        // Get the next error in the enumerator
        let mut error = None;
        let result = unsafe { errors.Next(&mut error) };
        if result == windows::ErrorCode::S_FALSE {
            break;
        }
        result.ok()?;
        let error = error.unwrap();

        // Get the start index and length of the error
        let mut start_index = 0u32;
        let mut length = 0u32;
        unsafe {
            error.get_StartIndex(&mut start_index).ok()?;
            error.get_Length(&mut length).ok()?;
        }

        // Get the substring from the ut8 encoded string
        let substring = &input[start_index as usize..(start_index + length) as usize];

        // Get the corrective action
        let mut action = intl::CORRECTIVE_ACTION::CORRECTIVE_ACTION_NONE;
        unsafe { error.get_CorrectiveAction(&mut action).ok()? };

        match action {
            intl::CORRECTIVE_ACTION::CORRECTIVE_ACTION_DELETE => {
                println!("Delete '{}'", substring);
            }
            intl::CORRECTIVE_ACTION::CORRECTIVE_ACTION_REPLACE => {
                // Get the replacement as a widestring and convert to a Rust String
                let mut replacement = PWSTR::default();
                unsafe { error.get_Replacement(&mut replacement).ok()? };

                println!("Replace: {} with {}", substring, unsafe {
                    read_to_string(replacement)
                });
            }
            intl::CORRECTIVE_ACTION::CORRECTIVE_ACTION_GET_SUGGESTIONS => {
                // Get an enumerator for all the suggestions for a substring
                let mut suggestions = None;
                unsafe { checker.Suggest(substring, &mut suggestions).ok()? };
                let suggestions = suggestions.unwrap();

                // Loop through the suggestions
                loop {
                    // Get the next suggestion breaking if the call to `Next` failed
                    let mut suggestion = PWSTR::default();
                    let result =
                        unsafe { suggestions.Next(1, &mut suggestion, std::ptr::null_mut()) };
                    if result == windows::ErrorCode::S_FALSE {
                        break;
                    }
                    result.ok()?;

                    println!("Maybe replace: {} with {}", substring, unsafe {
                        read_to_string(suggestion)
                    });
                }
            }
            _ => {}
        }
    }
    Ok(())
}

unsafe fn read_to_string(ptr: PWSTR) -> String {
    let mut len = 0usize;
    let mut cursor = ptr;
    loop {
        let val = cursor.0.read();
        if val == 0 {
            break;
        }
        len += 1;
        cursor = PWSTR(cursor.0.add(1));
    }

    let slice = std::slice::from_raw_parts(ptr.0, len);
    String::from_utf16(slice).unwrap()
}
