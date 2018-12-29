using System.Threading.Tasks;
using Microsoft.Toolkit.Services.MicrosoftGraph;

namespace YouthCenterSignIn
{
    class Graph
    {
        const string AppId = "6cd2d40f-e8d2-46be-ad3f-ac087d4b5284";
        readonly string[] Scopes = { "User.Read", "Contacts.ReadWrite" };

        public Graph()
        {
            MicrosoftGraphService instance = MicrosoftGraphService.Instance;
            instance.AuthenticationModel = MicrosoftGraphEnums.AuthenticationModel.V2;
            instance.Initialize(AppId,
                MicrosoftGraphEnums.ServicesToInitialize.UserProfile,
                Scopes);
        }

        public bool IsAuthenticated => MicrosoftGraphService.Instance.IsAuthenticated;

        public async Task<bool> Login() => await MicrosoftGraphService.Instance.TryLoginAsync();
    }
}
