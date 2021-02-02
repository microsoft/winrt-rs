use spellchecker_bindings::windows::win32;
use win32::{com, intl};
use windows::*;

fn main() -> windows::Result<()> {
    let input = std::env::args()
        .nth(1)
        .expect("Expected one command line argument for text to be spell-corrected");
    windows::initialize_mta()?;

    // Create ISpellCheckerFactory
    let factory: intl::ISpellCheckerFactory = windows::create_instance(&intl::SpellCheckerFactory)?;

    // Make sure that the "en-US" locale is supported
    let mut supported: BOOL = false.into();
    let locale: Vec<u16> = "en-US\0".encode_utf16().collect();
    // https://github.com/microsoft/windows-rs/issues/489
    factory
        .IsSupported(locale.as_ptr(), &mut supported.0)
        .ok()?;
    supported.expect("en-US is supported");

    // Create a ISpellChecker
    let mut checker = None;
    factory
        // https://github.com/microsoft/windows-rs/issues/486
        .CreateSpellChecker(locale.as_ptr(), &mut checker)
        .ok()?;
    let checker = checker.unwrap();

    // Get errors enumerator for the supplied string
    let mut errors = None;
    let text: Vec<u16> = input.encode_utf16().collect();
    // https://github.com/microsoft/windows-rs/issues/486
    checker.Check(text.as_ptr(), &mut errors).ok()?;
    let errors = errors.unwrap();

    // Loop through all the errors
    loop {
        // Get the next error in the enumerator
        let mut error = None;
        if errors.Next(&mut error) != windows::ErrorCode::S_OK {
            break;
        }
        let error = error.unwrap();

        // Get the start index and length of the error
        let mut start_index = 0u32;
        let mut length = 0u32;
        error.get_StartIndex(&mut start_index).ok()?;
        error.get_Length(&mut length).ok()?;

        // Get the substring from the ut8 encoded string
        let substring = &input[start_index as usize..(start_index + length) as usize];
        // Get the subtext from the wide string
        let mut subtext = text[start_index as usize..(start_index + length) as usize].to_vec();
        // Make sure the widestring ends in a null byte
        subtext.push(0);

        // Get the corrective action
        let mut action = intl::CORRECTIVE_ACTION::CORRECTIVE_ACTION_NONE;
        error.get_CorrectiveAction(&mut action).ok()?;

        // https://github.com/microsoft/windows-rs/issues/490
        if action == intl::CORRECTIVE_ACTION::CORRECTIVE_ACTION_DELETE {
            println!("Delete '{}'", substring);
        } else if action == intl::CORRECTIVE_ACTION::CORRECTIVE_ACTION_REPLACE {
            // Get the replacement as a widestring and convert to a Rust String
            let mut replacement: *mut u16 = std::ptr::null_mut();
            error.get_Replacement(&mut replacement).ok()?;
            let answer = unsafe { read_to_string(replacement) };

            println!("Replace: {} with {}", substring, answer);

            // Free the wide string's memory
            unsafe { com::CoTaskMemFree(replacement as _) };
        } else if action == intl::CORRECTIVE_ACTION::CORRECTIVE_ACTION_GET_SUGGESTIONS {
            // Get an enumerator for all the suggestions for a substring
            let mut suggestions = None;
            checker.Suggest(subtext.as_ptr(), &mut suggestions).ok()?;
            let suggestions = suggestions.unwrap();

            // Loop through the suggestions
            loop {
                // Get the next suggestion breaking if the call to `Next` failed
                let mut suggestion = std::ptr::null_mut();
                if suggestions.Next(1, &mut suggestion, std::ptr::null_mut())
                    != windows::ErrorCode::S_OK
                {
                    break;
                }

                // Convert the `suggestion` wide string to a Rust String
                let answer = unsafe { read_to_string(suggestion) };

                println!("Maybe replace: {} with {}", substring, answer);

                // Free the wide string's memory
                unsafe { com::CoTaskMemFree(suggestion as _) };
            }
        }
    }
    Ok(())
}

/// Assumes a valid pointer to a wide string
unsafe fn read_to_string(ptr: *const u16) -> String {
    let mut len = 0usize;
    let mut cursor = ptr;
    loop {
        let val = cursor.read();
        if val == 0 {
            break;
        }
        len += 1;
        cursor = cursor.add(1);
    }

    let slice = std::slice::from_raw_parts(ptr, len);
    String::from_utf16(slice).unwrap()
}
