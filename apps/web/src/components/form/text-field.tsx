import { Form, Input } from "antd";
import type { ComponentProps } from "react";

interface TextFieldProps extends Omit<ComponentProps<typeof Input>, "onChange"> {
    label: string;
    onChange: (value: string) => void;
    error?: string;
    description?: string;
    containerClassName?: string;
    id?: string;
}

export function TextField({
    id,
    label,
    value,
    onChange,
    error,
    description,
    containerClassName,
    ...props
}: TextFieldProps) {
    return (
        <Form.Item
            className={containerClassName}
            label={label}
            name={id}
            validateStatus={error ? "error" : ""}
            help={error ?? description}
            colon={false}
        >
            <Input
                id={id}
                value={value}
                onChange={(event) => onChange(event.target.value)}
                {...props}
            />
        </Form.Item>
    );
}
