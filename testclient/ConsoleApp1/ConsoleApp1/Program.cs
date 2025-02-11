using System;
using System.Net.Sockets;
using System.Text;

namespace TcpConsoleClient
{
  class Program
  {
    static void Main(string[] args)
    {
      try
      {
        // Establish a connection to localhost on port 7878.
        using (TcpClient client = new TcpClient("localhost", 7878))
        using (NetworkStream stream = client.GetStream())
        {
          Console.WriteLine("Connected to localhost:7878.");
          Console.WriteLine("Enter commands to send to the server (type 'exit' to quit).");

          while (true)
          {
            Console.Write("> ");
            // Read user input from the console.
            string command = Console.ReadLine();

            // If the user types "exit", break out of the loop.
            if (command.Equals("exit", StringComparison.OrdinalIgnoreCase))
            {
              break;
            }

            // Convert the command string to a byte array.
            byte[] commandBytes = Encoding.UTF8.GetBytes(command + "\n");
            // Send the command bytes to the server.
            stream.Write(commandBytes, 0, commandBytes.Length);

            // Prepare a buffer for reading the server's reply.
            byte[] buffer = new byte[1024];
            StringBuilder responseBuilder = new StringBuilder();

            // Read from the stream.
            // Note: In a production scenario you might need a more robust mechanism
            // for reading the complete reply (e.g. reading until a termination character).
            int bytesRead = stream.Read(buffer, 0, buffer.Length);
            if (bytesRead == 0)
            {
              Console.WriteLine("Server closed the connection.");
              break;
            }

            // Decode the received bytes into a string.
            responseBuilder.Append(Encoding.UTF8.GetString(buffer, 0, bytesRead));
            Console.WriteLine("Server reply: " + responseBuilder.ToString());
          }
        }
      }
      catch (Exception ex)
      {
        Console.WriteLine("Error: " + ex.Message);
      }
    }
  }
}