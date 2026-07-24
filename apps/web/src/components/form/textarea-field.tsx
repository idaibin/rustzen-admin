import { Form, Input } from "antd";
import type { ComponentProps } from "react";

interface TextareaFieldProps extends Omit<ComponentProps<typeof Input.TextArea>, "onChange"> {
    label: string;
    onChange: (value: string) => void;
    error?: string;
    description?: string;
    id?: string;
}

export function TextareaField({
    id,
    label,
    value,
    onChange,
    error,
    description,
    ...props
}: TextareaFieldProps) {
    return (
        <Form.Item
            label={label}
            name={id}
            validateStatus={error ? "error" : ""}
            help={error ?? description}
            colon={false}
        >
            <Input.TextArea
                id={id}
                value={value}
                onChange={(event) => onChange(event.target.value)}
                {...props}
            />
        </Form.Item>
    );
}
