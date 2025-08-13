package net.hydrius.pacts;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestInstance;

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class ValidationResultTest {

    @Test
    void testDefaultConstructor() {
        ValidationResult result = new ValidationResult();

        assertFalse(result.isValid());
        assertNull(result.getErrors());
    }

    @Test
    void testConstructorWithValidAndErrors() {
        List<String> errors = Arrays.asList("Error 1", "Error 2");
        ValidationResult result = new ValidationResult(false, errors);

        assertFalse(result.isValid());
        assertEquals(errors, result.getErrors());
    }

    @Test
    void testConstructorWithValidResult() {
        List<String> errors = new ArrayList<>();
        ValidationResult result = new ValidationResult(true, errors);

        assertTrue(result.isValid());
        assertEquals(errors, result.getErrors());
    }

    @Test
    void testSetAndGetValid() {
        ValidationResult result = new ValidationResult();
        result.setValid(true);

        assertTrue(result.isValid());
    }

    @Test
    void testSetAndGetErrors() {
        ValidationResult result = new ValidationResult();
        List<String> errors = Arrays.asList("Test error");
        result.setErrors(errors);

        assertEquals(errors, result.getErrors());
    }

    @Test
    void testHasErrorsWithErrors() {
        List<String> errors = Arrays.asList("Error 1", "Error 2");
        ValidationResult result = new ValidationResult(false, errors);

        assertTrue(result.hasErrors());
    }

    @Test
    void testHasErrorsWithoutErrors() {
        List<String> errors = new ArrayList<>();
        ValidationResult result = new ValidationResult(true, errors);

        assertFalse(result.hasErrors());
    }

    @Test
    void testHasErrorsWithNullErrors() {
        ValidationResult result = new ValidationResult();
        result.setErrors(null);

        assertFalse(result.hasErrors());
    }

    @Test
    void testGetErrorMessageWithErrors() {
        List<String> errors = Arrays.asList("Error 1", "Error 2", "Error 3");
        ValidationResult result = new ValidationResult(false, errors);

        assertEquals("Error 1; Error 2; Error 3", result.getErrorMessage());
    }

    @Test
    void testGetErrorMessageWithoutErrors() {
        List<String> errors = new ArrayList<>();
        ValidationResult result = new ValidationResult(true, errors);

        assertEquals("Validation successful", result.getErrorMessage());
    }

    @Test
    void testGetErrorMessageWithNullErrors() {
        ValidationResult result = new ValidationResult();
        result.setErrors(null);

        assertEquals("Validation successful", result.getErrorMessage());
    }

    @Test
    void testGetErrorMessageWithSingleError() {
        List<String> errors = Arrays.asList("Single error");
        ValidationResult result = new ValidationResult(false, errors);

        assertEquals("Single error", result.getErrorMessage());
    }

    @Test
    void testSetNullValues() {
        ValidationResult result = new ValidationResult(true, Arrays.asList("test"));

        result.setValid(false);
        result.setErrors(null);

        assertFalse(result.isValid());
        assertNull(result.getErrors());
    }

    @Test
    void testWithEmptyErrorList() {
        List<String> errors = new ArrayList<>();
        ValidationResult result = new ValidationResult(true, errors);

        assertTrue(result.isValid());
        assertTrue(result.getErrors().isEmpty());
        assertFalse(result.hasErrors());
        assertEquals("Validation successful", result.getErrorMessage());
    }

    @Test
    void testWithSpecialCharactersInErrors() {
        List<String> errors = Arrays.asList(
                "Error with special chars: !@#$%^&*()",
                "Error with unicode: 测试错误",
                "Error with quotes: \"quoted error\""
        );
        ValidationResult result = new ValidationResult(false, errors);

        assertFalse(result.isValid());
        assertTrue(result.hasErrors());
        assertEquals(3, result.getErrors().size());
        assertTrue(result.getErrorMessage().contains("Error with special chars"));
        assertTrue(result.getErrorMessage().contains("测试错误"));
        assertTrue(result.getErrorMessage().contains("quoted error"));
    }

    @Test
    void testValidResultWithNoErrors() {
        ValidationResult result = new ValidationResult(true, new ArrayList<>());

        assertTrue(result.isValid());
        assertFalse(result.hasErrors());
        assertEquals("Validation successful", result.getErrorMessage());
    }

    @Test
    void testInvalidResultWithNoErrors() {
        ValidationResult result = new ValidationResult(false, new ArrayList<>());

        assertFalse(result.isValid());
        assertFalse(result.hasErrors());
        assertEquals("Validation successful", result.getErrorMessage());
    }
}
