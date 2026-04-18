import { useState, type ReactNode } from "react";

interface DropDownMenuProps {
    subClassName: string;
    title: string;
    children: ReactNode;
}

export function DropDownMenu({
    subClassName,
    title,
    children,
}: DropDownMenuProps) {
    const [isOpened, setIsOpened] = useState(false);

    return (
        <div className={`dropdown ${subClassName} ${isOpened ? "opened" : "closed"}`}>
            <div
                title={`Cliquer pour ${isOpened ? "réduire" : "développer"}`}
                className={`dropdown-lid ${subClassName}-lid`}
                onClick={() => {
                    if (isOpened) {
                        setIsOpened(false);
                    } else {
                        setIsOpened(true);
                    }
                }}
            >
                <p>{title}</p>
            </div>
            <div
                className={`dropdown-content ${subClassName}-content`}
            >
                {children}
            </div>
        </div>
    );
}
