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
package net.hydrius.pacts;

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

    public boolean isValid() {
        return valid;
    }

    public void setValid(boolean valid) {
        this.valid = valid;
    }

    public List<String> getErrors() {
        return errors;
    }

    public void setErrors(List<String> errors) {
        this.errors = errors;
    }

    public boolean hasErrors() {
        return errors != null && !errors.isEmpty();
    }

    public String getErrorMessage() {
        if (errors == null || errors.isEmpty()) {
            return "Validation successful";
        }
        return String.join("; ", errors);
    }
}
