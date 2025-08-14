/*
 * Copyright © 2025 Hydrius, Project Hydrius, Wyrmlings
 * https://github.com/Project-Hydrius
 *
 * All rights reserved.
 *
 * This source code is part of the organizations named above.
 * Licensed for private use only. Unauthorized copying, modification,
 * or distribution is strictly prohibited.
 */
package net.hydrius.pacts.core;

import java.util.List;

/**
 * ValidationResult class that holds the result of a validation operation
 */
public class ValidationResult {

    private boolean valid;
    private List<String> errors;

    public ValidationResult() {
    }

    public ValidationResult(boolean valid, List<String> errors) {
        this.valid = valid;
        this.errors = errors;
    }

    /**
     * Checks if the validation was successful
     *
     * @return true if the validation was successful, false otherwise
     */
    public boolean isValid() {
        return valid;
    }

    /**
     * Sets the validation result
     *
     * @param valid true if the validation was successful, false otherwise
     */
    public void setValid(boolean valid) {
        this.valid = valid;
    }

    /**
     * Gets the list of errors
     *
     * @return the list of errors
     */
    public List<String> getErrors() {
        return errors;
    }

    /**
     * Sets the list of errors
     *
     * @param errors the list of errors
     */
    public void setErrors(List<String> errors) {
        this.errors = errors;
    }

    /**
     * Checks if there are any errors
     *
     * @return true if there are any errors, false otherwise
     */
    public boolean hasErrors() {
        return errors != null && !errors.isEmpty();
    }

    /**
     * Gets the error message
     *
     * @return the error message
     */
    public String getErrorMessage() {
        if (errors == null || errors.isEmpty()) {
            return "Validation successful";
        }
        return String.join("; ", errors);
    }
}
