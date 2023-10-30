'use client';
import React from 'react';
import Modal from 'react-modal';
import classnames from 'classnames';

interface IModalWrapperProps {
  contentStyle: 'center' | 'columns';
  isOpen: boolean;
  closeModal: () => void;
  children: any;
  className?: string;
  hideCloseButton?: boolean;
}

const ModalWrapper: React.FC<IModalWrapperProps> = ({
  contentStyle,
  isOpen,
  closeModal,
  children,
  className,
  hideCloseButton,
}) => {
  return (
    <Modal
      isOpen={isOpen}
      className={classnames(
        `${contentStyle === 'columns'
          ? 'flex justify-start items-center'
          : 'flex flex-col justify-start items-start'
        } bg-[#16161E] rounded-md relative focus:outline-none`,
        className,
      )}
      overlayClassName="z-50 fixed top-0 left-0 flex justify-center items-center bg-black/60 w-full h-full"
      onRequestClose={closeModal}
    >
      {hideCloseButton ? (
        ''
      ) : (
        <button
          className="absolute right-2 top-2"
          style={{ color: 'white' }}
          onClick={closeModal}
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="currentColor"
            className="w-6 h-6"
          >
            <path
              fillRule="evenodd"
              d="M5.47 5.47a.75.75 0 011.06 0L12 10.94l5.47-5.47a.75.75 0 111.06 1.06L13.06 12l5.47 5.47a.75.75 0 11-1.06 1.06L12 13.06l-5.47 5.47a.75.75 0 01-1.06-1.06L10.94 12 5.47 6.53a.75.75 0 010-1.06z"
              clipRule="evenodd"
            />
          </svg>
        </button>
      )}

      {/* {renderModalContent()} */}
      {children}
    </Modal>
  );
};

export default ModalWrapper;
