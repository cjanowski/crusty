import React from 'react';
import {
    Box,
    Container,
    Heading,
    Button,
    Flex,
    useColorMode,
} from '@chakra-ui/react';
import { MediaUpload } from '../Media/MediaUpload';
import { MediaGrid } from '../Media/MediaGrid';
import { useAuth } from '../../contexts/AuthContext';
import { useNavigate } from 'react-router-dom';

export const Dashboard: React.FC = () => {
    const { logout } = useAuth();
    const navigate = useNavigate();
    const { colorMode, toggleColorMode } = useColorMode();

    const handleLogout = () => {
        logout();
        navigate('/login');
    };

    return (
        <Box minH="100vh" bg={colorMode === 'dark' ? 'gray.800' : 'gray.50'}>
            <Flex
                as="header"
                bg={colorMode === 'dark' ? 'gray.700' : 'white'}
                px={4}
                py={2}
                shadow="sm"
                justifyContent="space-between"
                alignItems="center"
            >
                <Heading size="md" color={colorMode === 'dark' ? 'white' : 'gray.800'}>
                    Media Storage
                </Heading>
                <Flex gap={4}>
                    <Button onClick={toggleColorMode}>
                        {colorMode === 'light' ? 'Dark' : 'Light'} Mode
                    </Button>
                    <Button onClick={handleLogout} variant="outline">
                        Logout
                    </Button>
                </Flex>
            </Flex>

            <Container maxW="container.xl" py={8}>
                <MediaUpload />
                <Box mt={8}>
                    <MediaGrid />
                </Box>
            </Container>
        </Box>
    );
}; 